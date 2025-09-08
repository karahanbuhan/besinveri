CREATE TABLE IF NOT EXISTS food_tags (
    food_id     INTEGER NOT NULL,
    tag_id      INTEGER NOT NULL,
    PRIMARY KEY (food_id, tag_id)
)