CREATE TABLE activities (
    id BIGSERIAL PRIMARY KEY,
    data JSONB NOT NULL,
    created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    actor_uri VARCHAR NOT NULL
);

CREATE TABLE actors (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR,
    password VARCHAR,

    actor_uri VARCHAR NOT NULL,
    username VARCHAR,
    preferred_username VARCHAR NOT NULL,
    summary TEXT,
    inbox VARCHAR,
    icon VARCHAR,

    keys JSONB NOT NULL,
    created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    local BOOLEAN NOT NULL DEFAULT FALSE,

    UNIQUE(actor_uri),
    UNIQUE(email)
);

CREATE OR REPLACE FUNCTION set_updated_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.modified = now();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER set_actor_updated BEFORE UPDATE ON actors FOR EACH ROW EXECUTE PROCEDURE set_updated_timestamp();
CREATE TRIGGER set_activity_updated BEFORE UPDATE ON activities FOR EACH ROW EXECUTE PROCEDURE set_updated_timestamp();
