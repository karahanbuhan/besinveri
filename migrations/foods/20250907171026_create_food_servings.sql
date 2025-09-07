CREATE TABLE IF NOT EXISTS food_servings (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    food_id                 INTEGER NOT NULL,
    serving_description_id  INTEGER NOT NULL,
    weight                  REAL NOT NULL
)