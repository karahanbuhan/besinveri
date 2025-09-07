CREATE TABLE IF NOT EXISTS food_images (
    id                      INT PRIMARY KEY AUTOINCREMENT NOT NULL,
    image_url               TEXT NOT NULL UNIQUE
)