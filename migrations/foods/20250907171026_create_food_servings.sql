CREATE TABLE IF NOT EXISTS food_servings (
    food_id                 INTEGER NOT NULL,
    serving_description_id  INTEGER NOT NULL,
    weight                  REAL NOT NULL,
    PRIMARY KEY (food_id, serving_description_id)
)