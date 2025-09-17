use std::fs;

use crate::core::{food::Food, str::to_lower_en_kebab_case};
use anyhow::{Context, Error, anyhow};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use tracing::{info, warn};

pub(crate) async fn connect() -> Result<Pool<Sqlite>, Error> {
    // Veritabanı olarak SQLite kullanıyoruz, db/foods.sqlite dizininde olacak şekilde
    fs::create_dir_all("db").expect("db/ dizini oluşturulamadı");
    let database_url = "sqlite:db/foods.sqlite?mode=rwc"; // rwc mod sayesinde eğer veritabanı dosyası yoksa oluşturuyoruz
    let pool = SqlitePool::connect(database_url)
        .await
        .context("Veritabanına bağlanılamadı!")?;
    info!("Veritabanına bağlanıldı!");

    // Migration script'lerini çalıştırıyoruz, normalizasyon amaçlı birkaç tablo kullanıyoruz, /migrations/foods klasörünü inceleyebilirsiniz tabloları görmek için
    sqlx::migrate!("./migrations/foods")
        .run(&pool)
        .await
        .context("Migration'lar uygulanamadı!")?;
    info!("Migration'lar uygulandı!");

    // JSON dosyalarını bulup hepsini veritabanına eğer mevcut değillerse ekliyoruz. Bu sayede toplu şekilde veritabanına kolayca ekleme yapabiliriz
    // Ayrıca veritabanı dosyası .gitignore'da olacağı ve üzerine JSON harici eklemeler yapılacağı için; varsayılan JSON dosyalarının depoda olması yığın eklemeleri kolaylaştıracaktır
    // *DİKKAT* JSON okuma methodumuz async değil, bu kod sadece bağlantıda yani ilk açılışta çalıştırıldığı için main thread'i bloklamak sorun olmayacaktır
    if let Ok(foods) = load_foods_from_jsons("./db/foods") {
        // Eğer yoklar ise bu yemekleri veritabanına eklemeliyiz
        for food in foods {
            let food_name = food.description.to_owned();

            match insert_food(&pool, food).await {
                Ok(updated_food) => {
                    if let Some(food_id) = updated_food.id {
                        info!(
                            "{} başarıyla {} ID'si ile JSON dosyasından, veritabanına eklendi.",
                            food_name, food_id
                        );
                    } else {
                        // Bu hatanın hiçbir zaman oluşmaması gerek, yine de önlemimizi alalım
                        warn!(
                            "{} yemeği veritabanına eklendi ama ID'si alınamadı, kritik hata!",
                            food_name
                        );
                    }
                }
                Err(e) => {
                    warn!(
                        "{} yemeğini JSON dosyasından veritabanına aktarırken bir sorun oluştu: {}",
                        food_name, e
                    );
                }
            }
        }
    }

    Ok(pool)
}

async fn food_exists_by_description(pool: &SqlitePool, description: &str) -> Result<bool, Error> {
    Ok(
        sqlx::query_scalar::<_, i32>("SELECT id FROM foods WHERE description = ?")
            .bind(description)
            .fetch_optional(pool)
            .await?
            .is_some(),
    )
}

async fn insert_food(pool: &SqlitePool, food: Food) -> Result<Food, Error> {
    // Yemek halihazırda mevcutsa devam etmeye gerek yok, güncelleme için başka bir method kullanılacak
    if food_exists_by_description(pool, &food.description).await? {
        return Err(anyhow!(
            "{} isimli yemek zaten veritabanında mevcut, ekleme işlemi atlanıyor.",
            food.description
        ));
    }

    let mut tx = pool.begin().await?;

    // Resim ve kaynak için veri açılmadıysa açmamız ve id'yi almamız gerek
    sqlx::query("INSERT OR IGNORE INTO food_sources (description) VALUES (?)")
        .bind(&food.source)
        .execute(&mut *tx)
        .await?;
    let source_id =
        sqlx::query_scalar::<_, i32>("SELECT id FROM food_sources WHERE description = ? LIMIT 1")
            .bind(&food.source)
            .fetch_one(&mut *tx)
            .await?;

    sqlx::query("INSERT OR IGNORE INTO food_images (image_url) VALUES (?)")
        .bind(&food.image_url)
        .execute(&mut *tx)
        .await?;
    let image_id =
        sqlx::query_scalar::<_, i32>("SELECT id FROM food_images WHERE image_url = ? LIMIT 1")
            .bind(&food.image_url)
            .fetch_one(&mut *tx)
            .await?;

    // Resim ve kaynak id'leri yeni bir yemek eklemek için yeterli olacak

    // Etiketler ve alerjenler liste olduğu için kendi tabloları var, altta onu da ayarlayacağız. Önce yemek id'sine ihtiyacımız var

    // Upsert kullanmıyoruz, yani JSON verileri sadece varsayılan olarak kullanılıyor. Daha sonra manuel veritabanı üzerinden
    // değişiklik yapıldığı takdirde, JSON verilerinin üzerine yazılabilecek.

    // created_at ve updated_at değerlerini SQLite kendisi varsayılan vereceği için buradan müdahale etmiyoruz
    let food_id = sqlx
        ::query_scalar::<_, i32>(
            "INSERT OR IGNORE INTO foods (
            slug, description, verified, image_id, source_id, glycemic_index, energy, carbohydrate, protein, fat, saturated_fat, 
            trans_fat, sugar, cholesterol, sodium, potassium, iron, magnesium, calcium, zinc, vitamin_a, vitamin_b6, 
            vitamin_b12, vitamin_c, vitamin_d, vitamin_e, vitamin_k)

            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            
            RETURNING ID"
        )
        .bind(to_lower_en_kebab_case(&food.description))
        .bind(&food.description)
        .bind(food.verified.unwrap_or(false) as i32)
        .bind(&image_id)
        .bind(&source_id)
        .bind(&food.glycemic_index)
        .bind(&food.energy)
        .bind(&food.carbohydrate)
        .bind(&food.protein)
        .bind(&food.fat)
        .bind(&food.saturated_fat)
        .bind(&food.trans_fat)
        .bind(&food.sugar)
        .bind(&food.cholesterol)
        .bind(&food.sodium)
        .bind(&food.potassium)
        .bind(&food.iron)
        .bind(&food.magnesium)
        .bind(&food.calcium)
        .bind(&food.zinc)
        .bind(&food.vitamin_a)
        .bind(&food.vitamin_b6)
        .bind(&food.vitamin_b12)
        .bind(&food.vitamin_c)
        .bind(&food.vitamin_d)
        .bind(&food.vitamin_e)
        .bind(&food.vitamin_k)
        .fetch_one(&mut *tx).await?;

    // Her tag var mı kontrol edeceğiz, varsa da id'lerini yemekle eşleştirmek için food_tags'e ekleyeceğiz
    // Aynı normalizasyonu alerjenler için de yapacağız.
    // * ÖNEMLİ * Etiket ve alerjenler, standart bir kümelendirme olması için tamamen küçük harfler ile kaydedilecektir
    for tag in &food.tags {
        sqlx::query("INSERT OR IGNORE INTO tags (description) VALUES (LOWER(?))")
            .bind(&tag)
            .execute(&mut *tx)
            .await?;
        let tag_id = sqlx::query_scalar::<_, i32>(
            "SELECT id FROM tags WHERE description = LOWER(?) LIMIT 1",
        )
        .bind(&tag)
        .fetch_one(&mut *tx)
        .await?;

        // Şimdi de food_id <-> tag_id olarak birbirine eşleyeceğiz
        sqlx::query("INSERT OR IGNORE INTO food_tags (food_id, tag_id) VALUES (?, ?)")
            .bind(&food_id)
            .bind(&tag_id)
            .execute(&mut *tx)
            .await?;
    }

    // Aynı şekilde alerjenleri de ekliyoruz, tamamen küçük harf olacak alerjenlerin açıklaması da
    for allergen in &food.allergens {
        sqlx::query("INSERT OR IGNORE INTO allergens (description) VALUES (LOWER(?))")
            .bind(&allergen)
            .execute(&mut *tx)
            .await?;
        let allergen_id = sqlx::query_scalar::<_, i32>(
            "SELECT id FROM allergens WHERE description = LOWER(?) LIMIT 1",
        )
        .bind(&allergen)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query("INSERT OR IGNORE INTO food_allergens (food_id, allergen_id) VALUES (?, ?)")
            .bind(&food_id)
            .bind(&allergen_id)
            .execute(&mut *tx)
            .await?;
    }

    // Son olarak porsiyonlarını da kaydetmemiz gerek, her yemeğin farklı porsiyonları ve gramajları mevcut
    // Burada da aynı şekilde açıklama kısmı için normalizasyon yapıyoruz çünkü 'Porsiyon (Orta)' gibi açıklamaları birkaç defa kaydetmek istemiyoruz
    for serving in &food.servings {
        sqlx::query("INSERT OR IGNORE INTO serving_descriptions (description) VALUES (?)")
            .bind(&serving.0)
            .execute(&mut *tx)
            .await?;
        let serving_description_id = sqlx::query_scalar::<_, i32>(
            "SELECT id FROM serving_descriptions WHERE description = ? LIMIT 1",
        )
        .bind(&serving.0)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query("INSERT OR IGNORE INTO food_servings (food_id, serving_description_id, weight) VALUES (?, ?, ?)")
        .bind(&food_id)
        .bind(&serving_description_id)
        .bind(serving.1)
        .execute(&mut *tx)
        .await?;
    }

    // Transaction'ı tamamlayalım
    tx.commit().await?;

    // Yeni yemek yapısını döndürüyoruz, tabii ki veritabanı ID'si ile
    Ok(Food {
        id: Some(food_id as u32),
        ..food
    })
}

fn load_foods_from_jsons(dir: &str) -> Result<Vec<Food>, Error> {
    let mut all_foods: Vec<Food> = Vec::new();

    let paths = fs::read_dir(dir)?;
    for path in paths {
        let Ok(path) = path else {
            warn!("{} dizinindeki bir dosya okunamadı.", dir);
            continue;
        };

        let file_name = path.file_name().to_str().unwrap_or("???").to_owned();

        let Ok(file) = fs::File::open(path.path()) else {
            warn!("{} dizinindeki {} dosyası açılamadı!", dir, file_name);
            continue;
        };

        if let Ok(mut foods) = serde_json::from_reader::<_, Vec<Food>>(file) {
            all_foods.append(&mut foods);
        } else {
            warn!(
                "{}/{} dosyası JSON yemek formatında okunamadı!",
                dir, file_name
            );
        };
    }

    Ok(all_foods)
}

pub(crate) async fn select_all_foods_descriptions(pool: &SqlitePool) -> Result<Vec<String>, Error> {
    let mut descriptions: Vec<String> = Vec::new();

    for row in sqlx::query("SELECT description FROM foods")
        .fetch_all(pool)
        .await?
    {
        descriptions.push(row.try_get("description")?);
    }

    Ok(descriptions)
}

#[cfg(test)]
mod tests {
    use super::*; // Üst scope'daki fonksiyonları kullan

    #[tokio::test]
    async fn test_connect_and_migrate() -> Result<(), Error> {
        // In-memory veritabanı ile test
        let _pool = SqlitePool::connect("sqlite::memory:").await?;
        let _db_pool = connect().await?; // Gerçek dosya tablosu ile test için yorum satırını kaldır
        info!("Veritabanı bağlantısı ve migration testi geçti.");
        Ok(())
    }

    #[tokio::test]
    async fn test_food_exists_by_description() -> Result<(), Error> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        sqlx::migrate!("./migrations/foods").run(&pool).await?;

        let food = Food {
            slug: "test-yemek".to_string(),
            description: "Test Yemek".to_string(),
            image_url: "/test.jpg".to_string(),
            source: "test_source".to_string(),
            tags: vec!["test".to_string()],
            allergens: vec![],
            servings: std::collections::HashMap::new(),
            glycemic_index: 50.0,
            energy: 100.0,
            carbohydrate: 20.0,
            protein: 5.0,
            fat: 2.0,
            saturated_fat: 1.0,
            trans_fat: 0.0,
            sugar: 10.0,
            fiber: 3.0,
            cholesterol: 0.0,
            sodium: 50.0,
            potassium: 200.0,
            iron: 1.0,
            magnesium: 30.0,
            calcium: 10.0,
            zinc: 0.5,
            vitamin_a: 0.1,
            vitamin_b6: 0.2,
            vitamin_b12: 0.0,
            vitamin_c: 5.0,
            vitamin_d: 0.0,
            vitamin_e: 0.1,
            vitamin_k: 0.05,
            verified: None,
            id: None,
        };

        insert_food(&pool, food).await?;
        let exists = food_exists_by_description(&pool, "Test Yemek").await?;
        assert!(exists);

        let not_exists = food_exists_by_description(&pool, "Nonexistent").await?;
        assert!(!not_exists);

        info!("food_exists_by_description testi geçti.");
        Ok(())
    }

    #[tokio::test]
    async fn test_insert_food_and_load_json() -> Result<(), Error> {
        // In-memory veritabanı
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        sqlx::migrate!("./migrations/foods").run(&pool).await?;

        // Test için geçici dizin oluştur
        let temp_dir = "./db/test_temp";
        fs::create_dir_all(temp_dir).unwrap();

        // Test JSON dosyası oluştur
        let test_json = r#"
            [
                {
                    "description": "JSON Test Yemek",
                    "image_url": "/json_test.jpg",
                    "source": "json_source",
                    "tags": ["json_tag"],
                    "allergens": [],
                    "servings": {"Porsiyon": 100},
                    "glycemic_index": 60.0,
                    "energy": 120.0,
                    "carbohydrate": 25.0,
                    "protein": 6.0,
                    "fat": 3.0,
                    "saturated_fat": 1.5,
                    "trans_fat": 0.0,
                    "sugar": 12.0,
                    "fiber": 4.0,
                    "cholesterol": 0.0,
                    "sodium": 60.0,
                    "potassium": 220.0,
                    "iron": 1.2,
                    "magnesium": 35.0,
                    "calcium": 12.0,
                    "zinc": 0.6,
                    "vitamin_a": 0.15,
                    "vitamin_b6": 0.25,
                    "vitamin_b12": 0.0,
                    "vitamin_c": 6.0,
                    "vitamin_d": 0.0,
                    "vitamin_e": 0.12,
                    "vitamin_k": 0.06
                }
            ]
        "#;
        fs::write(format!("{}/test.json", temp_dir), test_json).unwrap();

        // Sadece test dizininden yükle
        let foods = load_foods_from_jsons(temp_dir).unwrap();
        assert_eq!(foods.len(), 1, "Sadece bir yemek yüklenmeli"); // Diğer dosyaları eklemez
        let food = foods[0].clone();

        // Yemeği ekle
        let result = insert_food(&pool, food).await;
        assert!(result.is_ok(), "Yemek eklenemedi");
        let food_id = result.unwrap().id.unwrap();
        assert!(food_id > 0, "Geçerli bir ID olmalı");

        info!("insert_food ve load_foods_from_jsons testi geçti.");

        // Testten sonra dosyayı ve dizini sil
        fs::remove_file(format!("{}/test.json", temp_dir)).unwrap();
        fs::remove_dir(temp_dir).unwrap();

        Ok(())
    }

    #[tokio::test]
    async fn test_select_all_foods_descriptions() -> Result<(), Error> {
        // In-memory veritabanı
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        // Migration'ları çalıştır
        sqlx::migrate!("./migrations/foods").run(&pool).await?;

        // Test verisi ekle
        let food1 = Food {
            slug: "fuji-elma".to_string(),
            description: "Fuji Elma".to_string(),
            image_url: "/fuji-elma.jpg".to_string(),
            source: "test_source".to_string(),
            tags: vec!["meyve".to_string()],
            allergens: vec![],
            servings: std::collections::HashMap::new(),
            glycemic_index: 40.0,
            energy: 50.0,
            carbohydrate: 10.0,
            protein: 0.5,
            fat: 0.2,
            saturated_fat: 0.0,
            trans_fat: 0.0,
            sugar: 8.0,
            fiber: 2.0,
            cholesterol: 0.0,
            sodium: 1.0,
            potassium: 150.0,
            iron: 0.1,
            magnesium: 5.0,
            calcium: 6.0,
            zinc: 0.1,
            vitamin_a: 0.0,
            vitamin_b6: 0.0,
            vitamin_b12: 0.0,
            vitamin_c: 4.0,
            vitamin_d: 0.0,
            vitamin_e: 0.1,
            vitamin_k: 0.0,
            verified: None,
            id: None,
        };
        let food2 = Food {
            slug: "muz".to_string(),
            description: "Muz".to_string(),
            image_url: "/muz.jpg".to_string(),
            source: "test_source".to_string(),
            tags: vec!["meyve".to_string()],
            allergens: vec![],
            servings: std::collections::HashMap::new(),
            glycemic_index: 60.0,
            energy: 90.0,
            carbohydrate: 20.0,
            protein: 1.0,
            fat: 0.3,
            saturated_fat: 0.1,
            trans_fat: 0.0,
            sugar: 15.0,
            fiber: 3.0,
            cholesterol: 0.0,
            sodium: 1.0,
            potassium: 300.0,
            iron: 0.2,
            magnesium: 10.0,
            calcium: 5.0,
            zinc: 0.15,
            vitamin_a: 0.0,
            vitamin_b6: 0.3,
            vitamin_b12: 0.0,
            vitamin_c: 10.0,
            vitamin_d: 0.0,
            vitamin_e: 0.1,
            vitamin_k: 0.0,
            verified: None,
            id: None,
        };

        // Yemekleri ekle
        insert_food(&pool, food1).await?;
        insert_food(&pool, food2).await?;

        // Fonksiyonu çağır
        let result = select_all_foods_descriptions(&pool).await?;

        // Sonuçları doğrula
        assert_eq!(result.len(), 2, "İki yemek açıklaması bekleniyor");
        assert_eq!(
            result[0],
            ("Fuji Elma".to_string()),
            "İlk açıklama eşleşmiyor"
        );
        assert_eq!(result[1], ("Muz".to_string()), "İkinci açıklama eşleşmiyor");

        // Boş tablo testi
        sqlx::query("DELETE FROM foods").execute(&pool).await?;
        let empty_result = select_all_foods_descriptions(&pool).await?;
        assert!(
            empty_result.is_empty(),
            "Boş tablo için boş sonuç bekleniyor"
        );

        info!("select_all_foods_descriptons testi geçti.");
        Ok(())
    }

    #[tokio::test]
    async fn test_select_all_foods_descriptions_no_table() -> Result<(), Error> {
        // In-memory veritabanı
        let pool = SqlitePool::connect("sqlite::memory:").await?;

        // Migration'ları çalıştırmadan fonksiyonu çağır (tablo yok)
        let result = select_all_foods_descriptions(&pool).await;

        // Hata beklendiğini doğrula
        assert!(result.is_err(), "Tablo olmadığında hata bekleniyor");
        if let Err(err) = result {
            assert!(
                err.to_string().contains("no such table"),
                "Hata 'no such table' içermeli"
            );
        }

        info!("select_all_foods_descriptions tablo yok testi geçti.");
        Ok(())
    }
}
