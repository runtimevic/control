use super::namespace_id::NamespaceId;
use super::statechart_namespace::LoadStateMachineMessage;
use crate::app_state::SharedState;
use socketioxide::ParserConfig;
use socketioxide::extract::{Data, SocketRef, TryData};
use socketioxide::layer::SocketIoLayer;
use std::str::FromStr;
use std::sync::Arc;

pub async fn init_socketio(app_state: Arc<SharedState>) -> SocketIoLayer {
    // create
    let (socketio_layer, io) = socketioxide::SocketIoBuilder::new()
        .max_buffer_size(1024)
        .with_parser(ParserConfig::msgpack())
        .build_layer();

    // Clone app_state for the first handler
    let app_state_main = app_state.clone();

    // set the on connect handler for main namespace
    io.ns("/main", move |socket: SocketRef| {
        handle_socket_connection(socket, app_state_main.clone());
    });

    // Clone app_state for the second handler
    let app_state_machine = app_state.clone();

    if let Err(err) = io.dyn_ns(
        "/machine/{vendor}/{machine}/{serial}",
        move |socket: SocketRef| {
            handle_socket_connection(socket, app_state_machine.clone());
        },
    ) {
        tracing::error!("Failed to detect machine namespace: {}", err);
    }

    // Clone app_state for machine statechart namespaces
    let app_state_machine_statechart = app_state.clone();

    if let Err(err) = io.dyn_ns(
        "/machine/{vendor}/{machine}/{serial}/statechart",
        move |socket: SocketRef| {
            handle_machine_statechart_connection(socket, app_state_machine_statechart.clone());
        },
    ) {
        tracing::error!("Failed to detect machine statechart namespace: {}", err);
    }

    // Clone app_state for statechart namespace
    let app_state_statechart = app_state.clone();

    // Setup /statechart namespace for state machine visualization
    io.ns("/statechart", move |socket: SocketRef| {
        tracing::info!("🔵 Socket connected to /statechart: {:?}", socket.id);

        let room = smol::block_on(async {
            let namespaces = app_state_statechart
                .socketio_setup
                .namespaces
                .read()
                .await;
            let room = namespaces.statechart_namespace.clone();
            
            // Update API senders for global statechart namespace
            let api_machines = app_state_statechart.api_machines.lock().await;
            room.set_api_senders(api_machines.clone()).await;
            drop(api_machines);
            
            room
        });

        tracing::info!("📝 Registering loadStateMachine handler for socket {:?}", socket.id);
        
        let room_clone = room.clone();
        socket.on("loadStateMachine", move |socket: SocketRef, TryData(res): TryData<LoadStateMachineMessage>| {
            tracing::info!("🟡 loadStateMachine event received");
            match res {
                Ok(msg) => {
                    tracing::info!("✅ Parsed loadStateMachine: config_len={}, machine_id={:?}", 
                        msg.config.len(), msg.machine_id);
                    let room = room_clone.clone();
                    smol::spawn(async move {
                        room.on_load_state_machine(socket, Data(msg)).await;
                    })
                    .detach();
                }
                Err(e) => {
                    tracing::error!("❌ Error extracting loadStateMachine message: {:?}", e);
                }
            }
        });

        let room_clone = room.clone();
        socket.on("sendEvent", move |socket: SocketRef, msg| {
            tracing::info!("Received sendEvent message");
            let room = room_clone.clone();
            smol::spawn(async move {
                room.on_send_event(socket, msg).await;
            })
            .detach();
        });

        let room_clone = room.clone();
        socket.on_disconnect(move |socket: SocketRef| {
            let room = room_clone.clone();
            smol::spawn(async move {
                room.on_disconnect(socket).await;
            })
            .detach();
        });
    });

    // set the io to the app state
    let mut socketio_guard = app_state.socketio_setup.socketio.write().await;
    socketio_guard.replace(io);

    socketio_layer
}

fn handle_socket_connection(socket: SocketRef, app_state: Arc<SharedState>) {
    let namespace_id = match NamespaceId::from_str(socket.ns()) {
        Ok(namespace_id) => namespace_id,
        Err(err) => {
            tracing::error!("Failed to parse NamespaceId: {}", err);
            return;
        }
    };

    // Setup disconnection handler
    setup_disconnection(socket.clone(), namespace_id.clone(), app_state.clone());

    // Setup connection
    setup_connection(socket, namespace_id, app_state);
}

fn setup_disconnection(socket: SocketRef, namespace_id: NamespaceId, app_state: Arc<SharedState>) {
    socket.on_disconnect(move |socket: SocketRef| {
        let namespace_id = namespace_id.clone();
        let app_state = app_state.clone();

        // Spawn async task to avoid blocking
        smol::spawn(async move {
            tracing::info!(
                "Socket disconnected from namespace socket={:?} namespace={}",
                socket.id,
                namespace_id,
            );

            // write-lock to mutate namespaces
            let mut namespaces_guard = app_state.socketio_setup.namespaces.write().await;

            match namespaces_guard.apply_mut(namespace_id.clone()).await {
                Ok(namespace) => {
                    namespace.unsubscribe(socket.clone());
                    tracing::info!(
                        "Socket unsubscribed from namespace socket={:?} namespace={}",
                        socket.id,
                        namespace_id
                    );
                }
                Err(err) => {
                    tracing::info!(
                        "Failed to unsubscribe socket from namespace socket={:?} namespace={} err={:?}",
                        socket.id,
                        namespace_id,
                        err
                    );
                }
            }
            if let NamespaceId::Machine(ident) = namespace_id.clone() {
                    match app_state.clone().api_machines.lock().await.get(&ident) {
                        Some(sender) => {
                            let _ = sender.send(machines::MachineMessage::UnsubscribeNamespace).await;
                        },
                        None => tracing::info!("sender doesnt exist for: {}",ident),
                    };
                }else{

                }
        })
        .detach();
    });
}

fn setup_connection(socket: SocketRef, namespace_id: NamespaceId, app_state: Arc<SharedState>) {
    let socket_clone = socket.clone();
    let namespace_id_clone = namespace_id.clone();
    let app_state_clone = app_state.clone();
    smol::spawn(async move {
        let guard = app_state_clone.socketio_setup.namespaces.read().await;
        let socket_queue_tx =  guard.main_namespace.namespace.socket_queue_tx.clone();
        drop(guard);

        let mut namespaces_guard = app_state_clone.socketio_setup.namespaces.write().await;
        // Ensure machine namespace exists before applying
        if let NamespaceId::Machine(_) = namespace_id_clone {
            let map = &mut namespaces_guard.machine_namespaces;
            if !map.contains_key(&namespace_id_clone) {
                tracing::info!(
                    "Registering new machine namespace: {}",
                    namespace_id_clone
                );
                // Clone the sender from your main namespace
                // Now create the namespace
                let ns = control_core::socketio::namespace::Namespace::new(socket_queue_tx);
                map.insert(namespace_id_clone.clone(), ns);
            }
        }

        // Apply and subscribe the socket
        match  namespaces_guard
            .apply_mut(namespace_id_clone.clone())
            .await
        {
            Ok(namespace) => {
                namespace.subscribe(socket_clone.clone());
                namespace.reemit(socket_clone);

                if let NamespaceId::Machine(ident) = namespace_id_clone {
                    match app_state.clone().api_machines.lock().await.get(&ident) {
                        Some(sender) => {
                            tracing::info!("subscribing namespace to {}",ident);
                            let _ = sender.send(machines::MachineMessage::SubscribeNamespace(namespace.clone())).await;
                        },
                        None => tracing::info!("sender doesnt exist for: {}",ident),
                    };
                }else{

                }
                }
                Err(err) => {
                    tracing::warn!(
                        "Couldn't subscribe socket to namespace, disconnecting socket={:?} namespace={} error={:?}",
                        socket_clone.id,
                        namespace_id_clone,
                        err
                    );
                    let _ = socket_clone.disconnect();
                }
            }
        }
    ).detach();

    tracing::info!(
        "Socket connected to namespace socket={:?} namespace={}",
        socket.id,
        namespace_id,
    );
}

fn handle_machine_statechart_connection(socket: SocketRef, app_state: Arc<SharedState>) {
    let namespace_path = socket.ns();
   
    let namespace_id = match NamespaceId::from_str(namespace_path) {
        Ok(namespace_id) => namespace_id,
        Err(err) => {
            tracing::error!("Failed to parse MachineStateChart NamespaceId: {}", err);
            return;
        }
    };

    let machine_id = match namespace_id {
        NamespaceId::MachineStateChart(id) => id,
        _ => {
            tracing::error!("Expected MachineStateChart namespace, got: {}", namespace_id);
            return;
        }
    };

    tracing::info!(
        "Socket connected to machine statechart: socket={:?} machine={:?}",
        socket.id,
        machine_id
    );

    // Get or create StateChartRoom for this machine
    let room = smol::block_on(async {
        // Clone the Arc to avoid borrowing issues
        let machine_rooms_arc = {
            let namespaces = app_state
                .socketio_setup
                .namespaces
                .read()
                .await;
            namespaces.machine_statechart_rooms.clone()
        };

        let mut rooms = machine_rooms_arc.write().await;

        let room = rooms
            .entry(machine_id.clone())
            .or_insert_with(|| {
                tracing::info!("Creating new StateChartRoom for machine: {:?}", machine_id);
                super::statechart_namespace::StateChartRoom::new()
            })
            .clone();

        // Update API senders in the room
        let api_machines = app_state.api_machines.lock().await;
        room.set_api_senders(api_machines.clone()).await;
        drop(api_machines);

        room
    });

    // Setup handlers
    let room_clone = room.clone();
    socket.on("loadStateMachine", move |socket: SocketRef, TryData(res): TryData<LoadStateMachineMessage>| {
        tracing::info!("🟡 loadStateMachine event received for machine");
        match res {
            Ok(msg) => {
                tracing::info!("✅ Parsed loadStateMachine for machine: config_len={}, machine_id={:?}", 
                    msg.config.len(), msg.machine_id);
                let room = room_clone.clone();
                smol::spawn(async move {
                    room.on_load_state_machine(socket, Data(msg)).await;
                })
                .detach();
            }
            Err(e) => {
                tracing::error!("❌ Error extracting loadStateMachine for machine: {:?}", e);
            }
        }
    });

    let room_clone = room.clone();
    socket.on("sendEvent", move |socket: SocketRef, msg| {
        tracing::info!("Received sendEvent message for machine");
        let room = room_clone.clone();
        smol::spawn(async move {
            room.on_send_event(socket, msg).await;
        })
        .detach();
    });

    let room_clone = room.clone();
    socket.on_disconnect(move |socket: SocketRef| {
        let room = room_clone.clone();
        smol::spawn(async move {
            room.on_disconnect(socket).await;
        })
        .detach();
    });
}
