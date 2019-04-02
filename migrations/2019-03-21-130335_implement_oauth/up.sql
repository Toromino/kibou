CREATE TABLE oauth_applications (
    id BIGSERIAL PRIMARY KEY,
    client_name VARCHAR,
    client_id VARCHAR NOT NULL,
    client_secret VARCHAR NOT NULL,
    redirect_uris VARCHAR NOT NULL,
    scopes VARCHAR NOT NULL,
    website VARCHAR,
    created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE oauth_authorizations (
    id BIGSERIAL PRIMARY KEY,
    application BIGSERIAL NOT NULL,
    actor VARCHAR NOT NULL,
    code VARCHAR NOT NULL,
    created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    valid_until TIMESTAMP NOT NULL
);

CREATE TABLE oauth_tokens (
    id BIGSERIAL PRIMARY KEY,
    application BIGSERIAL NOT NULL,
    actor VARCHAR NOT NULL,
    access_token VARCHAR NOT NULL,
    refresh_token VARCHAR NOT NULL,
    created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    valid_until TIMESTAMP NOT NULL
);


CREATE OR REPLACE FUNCTION set_updated_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.modified = now();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER set_oauth_application_updated BEFORE UPDATE ON oauth_applications FOR EACH ROW EXECUTE PROCEDURE set_updated_timestamp();
CREATE TRIGGER set_oauth_oauth_authorization_updated BEFORE UPDATE ON oauth_authorizations FOR EACH ROW EXECUTE PROCEDURE set_updated_timestamp();
CREATE TRIGGER set_oauth_token_updated BEFORE UPDATE ON oauth_tokens FOR EACH ROW EXECUTE PROCEDURE set_updated_timestamp();
