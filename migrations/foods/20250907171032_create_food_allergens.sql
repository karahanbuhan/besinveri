CREATE TABLE IF NOT EXISTS food_allergens (
    food_id         INTEGER NOT NULL,
    allergen_id     INTEGER NOT NULL,
    PRIMARY KEY (food_id, allergen_id)
)