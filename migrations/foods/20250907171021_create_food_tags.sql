CREATE TABLE IF NOT EXISTS food_tags (
    id                      INT PRIMARY KEY AUTOINCREMENT NOT NULL,
    food_id                 INT NOT NULL,
    tag_id                  INT NOT NULL
)