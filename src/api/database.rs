use std::fs;

use sqlx::{Pool, Sqlite, SqlitePool};
use tracing::info;

pub(crate) async fn get() -> Pool<Sqlite> {
    // Veritabanı olarak SQLite kullanıyoruz, db/foods.sqlite dizininde olacak şekilde
    fs::create_dir_all("db").expect("db/ dizini oluşturulamadı");
    let database_url = "sqlite:db/foods.sqlite?mode=rwc"; // rwc mod sayesinde eğer veritabanı dosyası yoksa oluşturuyoruz
    let pool = SqlitePool::connect(database_url)
        .await
        .expect("Veritabanına bağlanılamadı!");
    info!("Veritabanına bağlanıldı!");
    
    // Migration script'lerini çalıştırıyoruz, normalizasyon amaçlı birkaç tablo kullanıyoruz, /migrations/foods klasörünü inceleyebilirsiniz tabloları görmek için
    sqlx::migrate!("./migrations/foods").run(&pool).await.expect("Migration'lar uygulanamadı!");
    info!("Migration'lar uygulandı!");

    // JSON dosyalarını bulup hepsini veritabanına eğer mevcut değillerse ekliyoruz. Bu sayede toplu şekilde veritabanına kolayca ekleme yapabiliriz
    // Ayrıca veritabanı dosyası .gitignore'da olacağı ve üzerine JSON harici eklemeler yapılacağı için de varsayılan JSON dosyalarının depoda gözükmesi geliştirme sürecini kolaylaştıracak
    
    // TODO

    pool
}
