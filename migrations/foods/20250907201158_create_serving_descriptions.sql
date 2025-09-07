CREATE TABLE IF NOT EXISTS serving_descriptions (
    id                      INT PRIMARY KEY AUTOINCREMENT NOT NULL,
    description             TEXT NOT NULL UNIQUE
)