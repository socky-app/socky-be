-- Add up migration script here

CREATE TABLE users (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,

    email TEXT COLLATE "case_insensitive", 
    username TEXT COLLATE "case_insensitive",
    full_name TEXT NOT NULL,
    password_hash TEXT,

    is_ghost BOOLEAN NOT NULL DEFAULT false,

    -- 1: standard, 2: support, 3: admin
    role SMALLINT DEFAULT 1 CHECK (role IN (1, 2, 3)),
    
    -- 1: active, 2: disabled, 3: pending, 4: locked
    status SMALLINT DEFAULT 3 CHECK (status IN (1, 2, 3, 4)), 
    
    last_login_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now(),
    deleted_at TIMESTAMP -- Soft delete timestamp
);

-- Partial Unique Indexes
CREATE UNIQUE INDEX idx_users_email ON users(email) WHERE is_ghost = false;
CREATE UNIQUE INDEX idx_users_username ON users(username) WHERE is_ghost = false;

-- Only indexes users in their deletion grace period (acting as a tiny queue for the background worker)
CREATE INDEX idx_users_deleted_users ON users(deleted_at) WHERE deleted_at IS NOT NULL AND is_ghost = false;

-- Index for quickly filtering ghost vs real users
CREATE INDEX idx_users_ghost ON users(is_ghost) WHERE is_ghost = true;

-- Attach the auto-update trigger
SELECT trigger_updated_at('users');
