use axum::{http::StatusCode, Json};

pub(crate) async fn get_foods_handler() -> Result<Json<String>, (StatusCode, String)> {
    // {food-id}: {currentURL}/foods/{food-id}
    Ok(Json("".to_owned()))
}