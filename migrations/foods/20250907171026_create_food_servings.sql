CREATE TABLE IF NOT EXISTS food_servings (
    id                      INT PRIMARY KEY AUTOINCREMENT NOT NULL,
    food_id                 INT NOT NULL,
    serving_description_id  INT NOT NULL,
    weight                  REAL NOT NULL
)