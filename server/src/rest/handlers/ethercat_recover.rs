use axum::{extract::State, http::Response};
use std::sync::Arc;

use crate::{
    app_state::{HotThreadMessage, SharedState},
    ethercat::setup::setup_loop,
    metrics::io::get_ethercat_iface,
    rest::util::ResponseUtil,
};

#[derive(serde::Serialize)]
pub struct RecoverResponse {
    pub ok: bool,
}

#[axum::debug_handler]
pub async fn post_ethercat_recover(
    State(_app_state): State<Arc<SharedState>>,
) -> Response<axum::body::Body> {
    // Recovery requires full server restart due to PDU storage being leaked
    tracing::warn!("EtherCAT recovery endpoint called, but recovery requires server restart");
    ResponseUtil::error("EtherCAT recovery requires server restart. Please restart the application.")
}
