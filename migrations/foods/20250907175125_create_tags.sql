CREATE TABLE IF NOT EXISTS tags (
    id                      INT PRIMARY KEY AUTOINCREMENT NOT NULL,
    description             TEXT NOT NULL UNIQUE
)