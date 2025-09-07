CREATE TABLE IF NOT EXISTS allergens (
    id                      INT PRIMARY KEY AUTOINCREMENT NOT NULL,
    description             TEXT NOT NULL UNIQUE
)