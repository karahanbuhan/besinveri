use std::collections::BTreeMap;

use axum::{extract::State, Json};

use crate::SharedState;

pub(crate) async fn endpoints(State(shared_state): State<SharedState>) -> Json<BTreeMap<&'static str, String>> {
    // Henüz test etmedim ama ne olur ne olmaz diye to_owned atıyorum birkaç ms olsa bile config'e blok atılmaması için
    let api_base_url = &shared_state.config.lock().await.api.base_url.to_owned();
    let mut endpoints: BTreeMap<&'static str, String> = BTreeMap::new();

    endpoints.insert("api_health_url", format!("{}/{}", &api_base_url, "health"));
    endpoints.insert(
        "list_all_foods_url",
        format!("{}/{}", &api_base_url, "foods/list"),
    );
    endpoints.insert("get_food_url", format!("{}/{}", api_base_url, "food/{slug}"));
    endpoints.insert(
        "search_food_url",
        format!(
            "{}/{}",
            api_base_url, "foods/search?q={query}&mode={description, tag}&limit={limit}"
        ),
    );
    endpoints.insert("show_all_tags", format!("{}/{}", api_base_url, "tags"));

    Json(endpoints)
}
