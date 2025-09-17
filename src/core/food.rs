use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct Food {
    // ID ve Verified değerleri JSON'dan yükleme yapılırken bulunmayabilir, okurken de bu struct'ı kullanacağımız için Option olarak kullanacağız
    pub(crate) id: Option<i64>,
    pub(crate) slug: String,
    pub(crate) description: String,
    pub(crate) verified: Option<bool>,
    pub(crate) image_url: String,
    pub(crate) source: String,
    pub(crate) tags: Vec<String>,
    pub(crate) allergens: Vec<String>,
    pub(crate) servings: HashMap<String, f64>,
    pub(crate) glycemic_index: f64,
    pub(crate) energy: f64,
    pub(crate) carbohydrate: f64,
    pub(crate) protein: f64,
    pub(crate) fat: f64,
    pub(crate) saturated_fat: f64,
    pub(crate) trans_fat: f64,
    pub(crate) sugar: f64,
    pub(crate) fiber: f64,
    pub(crate) cholesterol: f64,
    pub(crate) sodium: f64,
    pub(crate) potassium: f64,
    pub(crate) iron: f64,
    pub(crate) magnesium: f64,
    pub(crate) calcium: f64,
    pub(crate) zinc: f64,
    pub(crate) vitamin_a: f64,
    pub(crate) vitamin_b6: f64,
    pub(crate) vitamin_b12: f64,
    pub(crate) vitamin_c: f64,
    pub(crate) vitamin_d: f64,
    pub(crate) vitamin_e: f64,
    pub(crate) vitamin_k: f64,
}