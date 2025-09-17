use std::collections::BTreeMap;

use axum::{Json, extract::State, http::StatusCode};

use anyhow::Result;
use tracing::error;

use crate::{SharedState, api::database, core::str::to_lower_en_kebab_case};

// HashMap yerine BTreeMap kullanma sebebimiz, yemek isimlerini alfabetik sıralamak istememiz. HashMap kullansaydık her seferinde rastgele sıralama olacaktı
pub(crate) async fn get_foods_handler(
    State(shared_state): State<SharedState>,
) -> Result<Json<BTreeMap<String, String>>, (StatusCode, &'static str)> {
    let descriptions = database::select_all_foods_descriptions(&*shared_state.api_db.lock().await)
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
        descriptions
            .into_iter()
             // Önce Fuji Elma -> fuji-elma şekline çeviriyoruz, tr karakter varsa en yapıyoruz
            .map(|desc| to_lower_en_kebab_case(&desc))
            // Daha sonra fuji-elma: https://API_BASE.URL/foods/food1\n.../food2 şeklinde gösteriyoruz 
            .map(|desc| (desc.clone(), api_base_url.clone() + "/foods/" + &desc))
            .collect(),
    ))
}
