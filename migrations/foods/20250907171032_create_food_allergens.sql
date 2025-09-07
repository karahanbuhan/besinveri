CREATE TABLE IF NOT EXISTS food_allergens (
    id              INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    description     TEXT NOT NULL,
    allergen_id     INTEGER NOT NULL
)