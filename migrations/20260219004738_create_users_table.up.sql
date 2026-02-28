-- Add up migration script here

CREATE TABLE users (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,

    email TEXT COLLATE "case_insensitive" NOT NULL, 
    password_hash TEXT NOT NULL,

    -- 1: standard, 2: support, 3: admin
    role SMALLINT NOT NULL DEFAULT 1 CHECK (role IN (1, 2, 3)),
    
    -- 1: active, 2: disabled, 3: pending, 4: locked
    status SMALLINT NOT NULL DEFAULT 3 CHECK (status IN (1, 2, 3, 4)), 
    
    last_login_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now(),
    deleted_at TIMESTAMP -- Soft delete timestamp (NULL means active)
);

-- Partial Unique Indexes
CREATE UNIQUE INDEX idx_users_email ON users(email) WHERE deleted_at IS NULL;

-- Only indexes the tiny percentage of users who are actually deleted
CREATE INDEX idx_users_deleted_users ON users(deleted_at) WHERE deleted_at IS NOT NULL;

-- Attach the auto-update trigger
SELECT trigger_updated_at('users');
