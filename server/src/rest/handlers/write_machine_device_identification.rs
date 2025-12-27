use crate::{app_state::SharedState, rest::util::ResponseUtil};
use axum::{Json, extract::State, http::Response};
use crate::socketio::main_namespace::machines_event::MachineObj;
use crate::ethercat::setup::group_devices_by_identification;
use machines::machine_identification::{
    DeviceHardwareIdentificationEthercat, DeviceMachineIdentification,
};

use std::sync::Arc;

use super::mutation::MutationResponse;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct MachineDeviceInfoRequest {
    pub device_machine_identification: DeviceMachineIdentification,
    pub hardware_identification_ethercat: DeviceHardwareIdentificationEthercat,
}

#[axum::debug_handler]
pub async fn post_write_machine_device_identification(
    State(app_state): State<Arc<SharedState>>,
    Json(body): Json<MachineDeviceInfoRequest>,
) -> Response<axum::body::Body> {
    let res = app_state
        .rt_machine_creation_channel
        .send(crate::app_state::HotThreadMessage::WriteMachineDeviceInfo(
            body.clone(),
        ))
        .await;

    match res {
        Ok(_) => (),
        Err(e) => tracing::error!(
            "Failed to send HotThreadMessage::WriteMachineDeviceInfo {}",
            e
        ),
    }

    // Update the in-memory device identification
    let mut ethercat_meta_data = app_state.ethercat_meta_data.write().await;
    for meta in ethercat_meta_data.iter_mut() {
        if let machines::machine_identification::DeviceHardwareIdentification::Ethercat(ethercat) = &meta.device_identification.device_hardware_identification {
            if ethercat.subdevice_index == body.hardware_identification_ethercat.subdevice_index {
                meta.device_identification.device_machine_identification = Some(body.device_machine_identification.clone());
                break;
            }
        }
    }

    // Re-group devices and add machines
    let device_identifications: Vec<_> = ethercat_meta_data.iter().map(|meta| meta.device_identification.clone()).collect();
    let grouping = group_devices_by_identification(&device_identifications);
    let machine_objs: Vec<_> = grouping.device_groups.into_iter().map(|group: Vec<machines::machine_identification::DeviceIdentificationIdentified>| {
        let first_device = group.first().unwrap();
        MachineObj {
            machine_identification_unique: first_device.device_machine_identification.machine_identification_unique.clone(),
            error: None,
        }
    }).collect();

    // Remove existing machines that are being updated
    for machine_obj in &machine_objs {
        app_state.remove_machine(&machine_obj.machine_identification_unique).await;
    }

    app_state.add_machines_if_not_exists(machine_objs).await;

    // Send updated machines event
    app_state.send_machines_event().await;

    drop(ethercat_meta_data);

    // Re-send the ethercat devices event to update the frontend
    app_state.clone().send_ethercat_devices_event().await;

    // Re-calculate machines
    let ethercat_meta_data = app_state.ethercat_meta_data.read().await;
    let device_identifications: Vec<_> = ethercat_meta_data.iter().map(|meta| meta.device_identification.clone()).collect();
    let grouping = group_devices_by_identification(&device_identifications);
    let machine_objs: Vec<_> = grouping.device_groups.into_iter().map(|group: Vec<machines::machine_identification::DeviceIdentificationIdentified>| {
        let first = group.first().unwrap();
        MachineObj {
            machine_identification_unique: first.device_machine_identification.machine_identification_unique.clone(),
            error: None,
        }
    }).collect();
    app_state.add_machines_if_not_exists(machine_objs).await;

    ResponseUtil::ok(MutationResponse::success())
}
