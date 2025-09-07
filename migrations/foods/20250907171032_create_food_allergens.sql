CREATE TABLE IF NOT EXISTS food_allergens (
    id                      INT PRIMARY KEY AUTOINCREMENT NOT NULL,
    description             TEXT NOT NULL,
    allergen_id             INT NOT NULL
)