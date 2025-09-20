use std::collections::BTreeMap;

use axum::Json;

use crate::api::endpoints;

pub(crate) async fn get_endpoints(api_base_url: &str) -> Json<BTreeMap<&'static str, String>> {
    let mut endpoints: BTreeMap<&'static str, String> = BTreeMap::new();

    endpoints.insert("api_status_url", format!("{}/{}", &api_base_url, "status"));
    endpoints.insert(
        "list_all_foods_url",
        format!("{}/{}", &api_base_url, "foods"),
    );
    endpoints.insert("get_food_url", format!("{}/{}", api_base_url, "foods/{slug}"));
    endpoints.insert(
        "search_food_url",
        format!(
            "{}/{}",
            api_base_url, "search/foods?query={query}&mode={description, tag}&limit={limit}"
        ),
    );

    Json(endpoints)
}
