use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Food {
    // ID ve Verified değerleri JSON'dan yükleme yapılırken bulunmayabilir, okurken de bu struct'ı kullanacağımız için Option olarak kullanacağız
    pub(crate) id: Option<u32>,
    pub(crate) description: String,
    pub(crate) verified: Option<bool>,
    pub(crate) image_url: String,
    pub(crate) source: String,
    pub(crate) tags: Vec<String>,
    pub(crate) allergens: Vec<String>,
    pub(crate) servings: HashMap<String, f32>,
    pub(crate) glycemic_index: f32,
    pub(crate) energy: f32,
    pub(crate) carbohydrate: f32,
    pub(crate) protein: f32,
    pub(crate) fat: f32,
    pub(crate) saturated_fat: f32,
    pub(crate) trans_fat: f32,
    pub(crate) sugar: f32,
    pub(crate) fiber: f32,
    pub(crate) cholesterol: f32,
    pub(crate) sodium: f32,
    pub(crate) potassium: f32,
    pub(crate) iron: f32,
    pub(crate) magnesium: f32,
    pub(crate) calcium: f32,
    pub(crate) zinc: f32,
    pub(crate) vitamin_a: f32,
    pub(crate) vitamin_b6: f32,
    pub(crate) vitamin_b12: f32,
    pub(crate) vitamin_c: f32,
    pub(crate) vitamin_d: f32,
    pub(crate) vitamin_e: f32,
    pub(crate) vitamin_k: f32,
}
