use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use tracing::info;

use crate::{SharedState, api::database, core::food::Food};

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

        return Ok(Json(sort_foods_by_relevance(&foods, &params.query).await));
    }

    Err((StatusCode::BAD_REQUEST, "Geçersiz sorgu!"))
}

async fn sort_foods_by_relevance<'a>(foods: &'a Vec<Food>, query: &str) -> Vec<Food> {
    let query = query.to_lowercase();

    // (original_index, yemek ref, skor)
    let mut scored: Vec<(usize, &'a Food, u64)> = foods
        .iter()
        .enumerate()
        .filter_map(|(idx, food)| {
            let desc_lower = food.description.to_lowercase();

            // Öncelikle sıralarken prefix şeklinde eşleşenlere öncelik vereceğiz
            // Örneğin ka diye aratıldığında 0: K*ar*puz, 1: Porta*ka*l şeklinde sıralamak istiyoruz
            // Bunun için basit bir puanlama sistemi yapıp bu puanlara göre sort edeceğiz, her eşleşen karakter için 1 puan ekleyeceğiz
            if desc_lower.starts_with(&query) {
                return Some((idx, food, 20u64));
            }

            // Prefix kontrolünü hiç geçemeyen yemekler için, örneğin ka diye arattığımızda Porta*ka*l ve Ma*ka*rna makarnanın öncelikli olmasını istiyoruz
            // Başa ne kadar yakınsa o kadar yüksek puan olacak yani, pozisyona göre puan vereceğiz
            if let Some(pos) = desc_lower.find(&query) {
                let len = desc_lower.len();
                let score = 10 * (len.saturating_sub(pos)) / len.max(1);
                return Some((idx, food, score as u64));
            }

            None
        })
        .collect();

    // Puanlara göre yüksekten düşüğe sıralıyoruz
    scored.sort_unstable_by(|a, b| b.2.cmp(&a.2));

    // Referans döndürmek ne yazık ki Axum ile sorun oluyor o yüzden to_owned atacağız, optimize edilebilir
    scored
        .into_iter()
        .map(|(_, food, _)| food.to_owned())
        .collect()
}