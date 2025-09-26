use std::collections::BTreeMap;

use axum::Json;

pub(crate) async fn get_endpoints(api_base_url: &str) -> Json<BTreeMap<&'static str, String>> {
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

    Json(endpoints)
}
