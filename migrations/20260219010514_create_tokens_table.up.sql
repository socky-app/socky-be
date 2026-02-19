-- Add up migration script here

CREATE TABLE refresh_tokens (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    
    -- Foreign Key: Links to the users table. 
    -- ON DELETE CASCADE: If the user is deleted, their tokens vanish automatically.
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Maps to Rust Uuid. Postgres has a native 128-bit UUID type.
    -- Used for Refresh Token Rotation (detecting reuse).
    family_id UUID NOT NULL,
    
    -- Maps to Rust Vec<u8>. Efficient storage for hashes (e.g., SHA256).
    token_hash BYTEA NOT NULL,
    
    is_revoked BOOLEAN NOT NULL DEFAULT FALSE,
    
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);

-- Index for looking up tokens by their hash (critical for login/refresh)
CREATE INDEX idx_refresh_tokens_token_hash ON refresh_tokens(token_hash);

-- Index for finding all tokens in a family (critical for rotation reuse detection)
CREATE INDEX idx_refresh_tokens_family_id ON refresh_tokens(family_id);

-- Index for listing all tokens belonging to a specific user (for "Log out all devices")
CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);

-- Attach the auto-update trigger
SELECT trigger_updated_at('refresh_tokens');