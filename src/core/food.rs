use std::collections::{BTreeMap};

use serde::{Deserialize, Serialize};
use sqlx::{Error, FromRow, Row, sqlite::SqliteRow};
use tracing::info;

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub(crate) struct Food {
    // ID ve Verified değerleri JSON'dan yükleme yapılırken bulunmayabilir, okurken de bu struct'ı kullanacağımız için Option olarak kullanacağız
    pub(crate) id: Option<i64>,
    pub(crate) slug: Option<String>,
    pub(crate) description: String,
    pub(crate) verified: Option<bool>,
    pub(crate) image_url: String,
    pub(crate) source: String,
    pub(crate) tags: Vec<String>,
    pub(crate) allergens: Vec<String>,
    pub(crate) servings: BTreeMap<String, f64>,
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
    pub(crate) water: f64,
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

impl<'r> FromRow<'r, SqliteRow> for Food {
    fn from_row(row: &'r SqliteRow) -> Result<Self, Error> {
        // sqlx::Error kullandığımız için serde hatalarını çevirmemize yardımcı olacak bir closure ekleyelim
        let json_err = |e: serde_json::Error| Error::Decode(e.into());

        // SQL'de integer olarak tutuyoruz bu boolean'ı o yüzden çevirmemiz gerek
        let verified_int: i64 = row.try_get("verified")?;
        let verified = Some(verified_int != 0);

        // Bazı veriler liste döndürdüğü için JSON'a çeviriyorduk okurken, aynı şekilde açıyoruz
        let tags_str: String = row.try_get("tags")?;
        let tags = serde_json::from_str(&tags_str).map_err(json_err)?;

        let allergens_str: String = row.try_get("allergens")?;
        let allergens = serde_json::from_str(&allergens_str).map_err(json_err)?;

        let servings_str: String = row.try_get("servings")?;
        let servings = serde_json::from_str(&servings_str).map_err(json_err)?;

        // Son olarak struct'ımızı döndürüyoruz
        Ok(Food {
            id: Some(row.try_get("id")?),
            slug: row.try_get("slug")?,
            description: row.try_get("description")?,
            verified, 
            image_url: row.try_get("image_url")?,
            source: row.try_get("source_description")?,
            tags,
            allergens,
            servings,
            glycemic_index: row.try_get("glycemic_index")?,
            energy: row.try_get("energy")?,
            carbohydrate: row.try_get("carbohydrate")?,
            protein: row.try_get("protein")?,
            fat: row.try_get("fat")?,
            saturated_fat: row.try_get("saturated_fat")?,
            trans_fat: row.try_get("trans_fat")?,
            sugar: row.try_get("sugar")?,
            fiber: row.try_get("fiber")?,
            water: row.try_get("water")?,
            cholesterol: row.try_get("cholesterol")?,
            sodium: row.try_get("sodium")?,
            potassium: row.try_get("potassium")?,
            iron: row.try_get("iron")?,
            magnesium: row.try_get("magnesium")?,
            calcium: row.try_get("calcium")?,
            zinc: row.try_get("zinc")?,
            vitamin_a: row.try_get("vitamin_a")?,
            vitamin_b6: row.try_get("vitamin_b6")?,
            vitamin_b12: row.try_get("vitamin_b12")?,
            vitamin_c: row.try_get("vitamin_c")?,
            vitamin_d: row.try_get("vitamin_d")?,
            vitamin_e: row.try_get("vitamin_e")?,
            vitamin_k: row.try_get("vitamin_k")?,
        })
    }
}