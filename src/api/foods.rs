use axum::{Json, extract::State, http::StatusCode};

use anyhow::Result;
use tracing::error;

use crate::{SharedState, api::database};

pub(crate) async fn get_foods_handler(
    State(shared_state): State<SharedState>,
) -> Result<Json<String>, (StatusCode, &'static str)> {
    let descriptors = database::get_all_foods_descriptors(&*shared_state.api_db.lock().await)
        .await
        .map_err(|e| {
            error!("Veritabanı hatası: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Veritabanı yemek açıklama sorgusu hatası",
            )
        })?;

    // {food-id}: {currentURL}/foods/{food-id}
    
    Ok(Json("".to_owned()))
}
