CREATE TABLE IF NOT EXISTS serving_descriptions (
    id              INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    description     TEXT NOT NULL UNIQUE
)