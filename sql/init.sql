CREATE TABLE IF NOT EXISTS hooks
(
    user_id INTEGER,
    guild_id   INTEGER,
    hook    TEXT
);

CREATE INDEX hooks_by_user ON hooks (user_id);
CREATE INDEX hooks_by_guild ON hooks (guild_id);