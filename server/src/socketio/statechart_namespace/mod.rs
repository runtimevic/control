//! StateChart namespace for real-time state machine visualization and control
//!
//! This namespace provides Socket.IO handlers for:
//! - Loading XState JSON configurations
//! - Executing state machines with custom actions/guards
//! - Sending events to trigger transitions
//! - Real-time execution state updates to frontend

use control_core::state_machine::{ExecutionState, StateMachine};
use control_core::socketio::event::Event;
use serde::{Deserialize, Serialize};
use socketioxide::extract::{Data, SocketRef};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info, instrument, warn};
use machines::machine_identification::MachineIdentificationUnique;
use machines::{MachineMessage};
use smol::channel::Sender;

pub mod hardware_actions;

/// Message to load a new state machine from XState JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadStateMachineMessage {
    pub config: String, // XState JSON as string
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub machine_id: Option<MachineIdentificationUnique>, // Target machine for hardware actions
}

/// Message to send an event to the state machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendEventMessage {
    pub event: String,
}

/// Response after loading a state machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadStateMachineResponse {
    pub success: bool,
    pub message: String,
    pub execution_state: Option<ExecutionState>,
}

/// StateChart room managing state machines per socket connection
#[derive(Clone)]
pub struct StateChartRoom {
    /// Active state machines indexed by socket ID
    machines: Arc<smol::lock::Mutex<HashMap<String, StateMachine>>>,
    /// API senders for machines (indexed by MachineIdentificationUnique)
    api_senders: Arc<smol::lock::RwLock<HashMap<MachineIdentificationUnique, Sender<MachineMessage>>>>,
}

impl StateChartRoom {
    pub fn new() -> Self {
        Self {
            machines: Arc::new(smol::lock::Mutex::new(HashMap::new())),
            api_senders: Arc::new(smol::lock::RwLock::new(HashMap::new())),
        }
    }

    /// Set the API senders for machines
    pub async fn set_api_senders(&self, senders: HashMap<MachineIdentificationUnique, Sender<MachineMessage>>) {
        *self.api_senders.write().await = senders;
    }

    /// Helper to emit events in the standard format
    fn emit_event<T: Serialize + Send + Sync + Clone + 'static>(
        socket: &SocketRef,
        name: &str,
        data: T,
    ) -> Result<(), socketioxide::SendError> {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let event = Event {
            name: name.to_string(),
            data,
            ts,
        };

        socket.emit("event", &event)
    }

    /// Handler for loading a new state machine
    #[instrument(skip(self, socket))]
    pub async fn on_load_state_machine(
        &self,
        socket: SocketRef,
        Data(msg): Data<LoadStateMachineMessage>,
    ) {
        let socket_id = socket.id.to_string();
        info!("Loading state machine for socket {} with machine_id {:?}", socket_id, msg.machine_id);

        match StateMachine::from_json(&msg.config) {
            Ok(mut machine) => {
                info!("Successfully parsed state machine");
                
                // Register hardware-specific actions based on actionMappings
                if let Some(machine_id) = &msg.machine_id {
                    // Get API sender for this machine
                    let api_senders = self.api_senders.read().await;
                    if let Some(api_sender) = api_senders.get(machine_id) {
                        // Get action mappings from config and clone them
                        if let Some(mappings) = machine.config().action_mappings.clone() {
                            info!("Registering {} action mappings for machine {:?}", mappings.len(), machine_id);
                            let api_sender_clone = api_sender.clone();
                            let machine_id_clone = machine_id.clone();
                            drop(api_senders); // Release read lock
                            
                            for (action_name, mutation_json) in mappings {
                                tracing::info!(
                                    "[StateMachine] Registering action '{}' -> mutation: {:?}",
                                    action_name,
                                    mutation_json
                                );
                                let action = Arc::new(hardware_actions::MachineApiAction::new(
                                    action_name.clone(),
                                    machine_id_clone.clone(),
                                    mutation_json.clone(),
                                    api_sender_clone.clone(),
                                ));
                                machine.actions_mut().register(action);
                            }
                        } else {
                            warn!("No actionMappings found in state machine config");
                        }
                    } else {
                        error!("Machine {:?} not found in API senders", machine_id);
                    }
                } else {
                    // No machine_id: register mock actions for testing
                    info!("No machine_id provided, registering mock hardware actions");
                    hardware_actions::register_actions(machine.actions_mut());
                }
                
                hardware_actions::register_guards(machine.guards_mut());

                let exec_state = machine.execution_state();

                // Store machine
                self.machines.lock().await.insert(socket_id, machine);

                // Send success response
                let response = LoadStateMachineResponse {
                    success: true,
                    message: "State machine loaded successfully".to_string(),
                    execution_state: Some(exec_state.clone()),
                };

                if let Err(e) = Self::emit_event(&socket, "loadStateMachineResponse", response) {
                    error!("Failed to send loadStateMachineResponse: {:?}", e);
                }

                // Send initial execution state
                if let Err(e) = Self::emit_event(&socket, "executionState", exec_state) {
                    error!("Failed to send initial executionState: {:?}", e);
                }
            }
            Err(e) => {
                error!("Failed to parse state machine JSON: {:?}", e);
                let response = LoadStateMachineResponse {
                    success: false,
                    message: format!("Failed to parse JSON: {}", e),
                    execution_state: None,
                };

                if let Err(e) = Self::emit_event(&socket, "loadStateMachineResponse", response) {
                    error!("Failed to send error response: {:?}", e);
                }
            }
        }
    }

    /// Handler for sending an event to the state machine
    #[instrument(skip(self, socket))]
    pub async fn on_send_event(&self, socket: SocketRef, Data(msg): Data<SendEventMessage>) {
        let socket_id = socket.id.to_string();

        let mut machines = self.machines.lock().await;
        let machine = match machines.get_mut(&socket_id) {
            Some(m) => m,
            None => {
                warn!("No state machine found for socket {}", socket_id);
                return;
            }
        };

        match machine.send(&msg.event) {
            Ok(transitioned) => {
                if transitioned {
                    info!(
                        "Event '{}' triggered transition to '{}'",
                        msg.event,
                        machine.current_state()
                    );

                    // Send updated execution state
                    let exec_state = machine.execution_state();
                    if let Err(e) = Self::emit_event(&socket, "executionState", exec_state) {
                        error!("Failed to send executionState: {:?}", e);
                    }
                } else {
                    warn!(
                        "Event '{}' did not trigger a transition (blocked by guard or no matching transition)",
                        msg.event
                    );
                }
            }
            Err(e) => {
                error!("Failed to send event '{}': {:?}", msg.event, e);
            }
        }
    }

    /// Handler for socket disconnection - cleanup state machine
    #[instrument(skip(self, socket))]
    pub async fn on_disconnect(&self, socket: SocketRef) {
        let socket_id = socket.id.to_string();
        self.machines.lock().await.remove(&socket_id);
        info!("Removed state machine for disconnected socket {}", socket_id);
    }
}

impl Default for StateChartRoom {
    fn default() -> Self {
        Self::new()
    }
}
