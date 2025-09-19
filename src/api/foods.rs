use std::collections::BTreeMap;

use axum::{extract::{Path, State}, http::StatusCode, Json};

use anyhow::Result;
use tracing::error;

use crate::{SharedState, api::database, core::food::Food};

pub(crate) async fn get_food_handler(
    Path(slug): Path<String>,
    State(shared_state): State<SharedState>,
) -> Result<Json<Food>, (StatusCode, &'static str)> {
    let food = database::select_food_by_slug(&*shared_state.api_db.lock().await, &slug).await.map_err(|e| {
        error!("Veritabanı yemek bilgisi sorgularken hata oluştu: {:?}", e);
        (StatusCode::NOT_FOUND, "Bu yemekle ilgili veriye ulaşılamadı")
    })?;
    
        Ok(Json(food))
}

// HashMap yerine BTreeMap kullanma sebebimiz, yemek isimlerini alfabetik sıralamak istememiz. HashMap kullansaydık her seferinde rastgele sıralama olacaktı
pub(crate) async fn get_foods_handler(
    State(shared_state): State<SharedState>,
) -> Result<Json<BTreeMap<String, String>>, (StatusCode, &'static str)> {
    let slugs = database::select_all_foods_slugs(&*shared_state.api_db.lock().await)
        .await
        .map_err(|e| {
            error!(
                "Veritabanı yemek açıklamaları sorgularken hata oluştu: {:?}",
                e
            );
            (StatusCode::INTERNAL_SERVER_ERROR, "Veritabanı hatası")
        })?;

    let api_base_url = &shared_state.config.lock().await.api.base_url;

    Ok(Json(
        slugs
            .into_iter()
            .map(|slug| slug)
            // Daha sonra fuji-elma: https://API_BASE.URL/foods/food1\n.../food2 şeklinde gösteriyoruz
            .map(|slug| (slug.clone(), api_base_url.clone() + "/foods/" + &slug))
            .collect(),
    ))
}