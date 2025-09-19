use core::error;

use anyhow::anyhow;
use axum::{extract::{Query, State}, http::StatusCode, Json};
use serde::Deserialize;
use tracing::info;

use crate::{api::database, core::food::Food, SharedState};

#[derive(Deserialize)]
pub(crate) struct SearchParams {
    query: String,
    mode: String,
    limit: u64, 
}

pub(crate) async fn get_search_food_handler(
    params: Query<SearchParams>,
    State(shared_state): State<SharedState>,
) -> Result<Json<Vec<Food>>, (StatusCode, &'static str)> {
    if params.mode.to_lowercase().eq("description") || params.mode.to_lowercase().eq("name") {
        let foods = database::search_food_by_description_wild(&*shared_state.api_db.lock().await, &params.query, params.limit).await.map_err(|e| {
            info!("Açıklama/isim ile yemek ararken bir hata oluştu, parametreler: query={}&limit={}&mode={}\nHata: {}", params.query, params.mode, params.limit, e);
            (StatusCode::NOT_FOUND, "Yemek ararken sonuç bulunamadı")
        })?;
    }
    todo!()
}