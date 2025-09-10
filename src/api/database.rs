use std::fs;

use crate::core::food::Food;
use anyhow::{Context, Error, anyhow};
use sqlx::{Pool, Sqlite, SqlitePool};
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
    Ok(sqlx::query_scalar::<_, i32>("SELECT id FROM foods WHERE description = ?")
        .bind(description)
        .fetch_optional(pool)
        .await?.is_some())
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
            description, verified, image_id, source_id, glycemic_index, energy, carbohydrate, protein, fat, saturated_fat, 
            trans_fat, sugar, cholesterol, sodium, potassium, iron, magnesium, calcium, zinc, vitamin_a, vitamin_b6, 
            vitamin_b12, vitamin_c, vitamin_d, vitamin_e, vitamin_k)

            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            
            RETURNING ID"
        )
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
