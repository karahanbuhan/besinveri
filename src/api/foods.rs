use std::collections::BTreeMap;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};

use anyhow::Result;
use serde::Deserialize;
use tracing::error;

use crate::{SharedState, api::database, core::food::Food};

pub(crate) async fn get_food_handler(
    Path(slug): Path<String>,
    State(shared_state): State<SharedState>,
) -> Result<Json<Food>, (StatusCode, &'static str)> {
    let food = database::select_food_by_slug(&*shared_state.api_db.lock().await, slug)
        .await
        .map_err(|e| {
            error!("Veritabanı yemek bilgisi sorgularken hata oluştu: {:?}", e);
            (
                StatusCode::NOT_FOUND,
                "Bu yemekle ilgili veriye ulaşılamadı",
            )
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

#[derive(Deserialize)]
pub(crate) struct SearchParams {
    query: String,
    mode: String,
    limit: u64,
}

pub(crate) async fn get_foods_search_handler(
    params: Query<SearchParams>,
    State(shared_state): State<SharedState>,
) -> Result<Json<Vec<Food>>, (StatusCode, &'static str)> {
    // Moda göre uygun veritabanı sorgusunu atıyoruz
    let mode = params.mode.to_lowercase();

    let foods = match mode.as_str() {
        // İsim ile aratmada ayrıca sıralıyoruz benzerliğine göre
        "description" | "name" => Ok(sort_foods_by_relevance(&database::search_food_by_description_wild(&*shared_state.api_db.lock().await, &params.query, params.limit).await.map_err(|e| {
            error!("Açıklama/isim ile yemek ararken bir hata oluştu, parametreler: query={}&limit={}&mode={}\nHata: {}", params.query, params.mode, params.limit, e);
            (StatusCode::NOT_FOUND, "İsim ile yemek ararken sonuç bulunamadı")
        })?, &params.query).await),

        "tag" => Ok( database::search_food_by_tag_wild(
            &*shared_state.api_db.lock().await,
            &params.query,
            params.limit,
        )
        .await.map_err(|e| {
            error!("Etiket ile yemek ararken bir hata oluştu, parametreler: query={}&limit={}&mode={}\nHata: {}", params.query, params.mode, params.limit, e);
            (StatusCode::NOT_FOUND, "Etiket ile yemek ararken sonuç bulunamadı")
        })?),

        _ => Err((StatusCode::BAD_REQUEST, "Geçersiz sorgu!"))
    }?;

    Ok(Json(foods))
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
