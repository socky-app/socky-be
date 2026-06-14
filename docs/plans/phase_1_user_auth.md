# Phase 1: User Profile Extensions & Authorization Logic

This document specifies the database schema changes, codebase modifications, and API endpoints required to implement Phase 1.

---

## 1. Database Schema Changes

Since the application is still in development with no production data, we will modify the existing migration file [20260219004738_create_users_table.up.sql](file:///home/schneider/Documents/socky-be/migrations/20260219004738_create_users_table.up.sql) directly instead of creating a new migration.

### Changes to the `users` table

Add the following columns to the `CREATE TABLE users` statement:

```sql
username TEXT COLLATE "case_insensitive",
full_name TEXT,
is_ghost BOOLEAN NOT NULL DEFAULT false,
```

Additionally, `email` and `password_hash` must become nullable to support ghost users (who have no credentials):

```sql
email TEXT COLLATE "case_insensitive",  -- was NOT NULL
password_hash TEXT,                      -- was NOT NULL
```

Add the following partial unique index after the existing indexes:

```sql
-- Partial unique index for usernames (allows NULL, unique among active users)
CREATE UNIQUE INDEX idx_users_username ON users(username) WHERE deleted_at IS NULL AND username IS NOT NULL;
```

### Resulting migration (complete)

```sql
CREATE TABLE users (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,

    email TEXT COLLATE "case_insensitive",
    username TEXT COLLATE "case_insensitive",
    full_name TEXT,
    password_hash TEXT,

    is_ghost BOOLEAN NOT NULL DEFAULT false,

    -- 1: standard, 2: support, 3: admin
    role SMALLINT NOT NULL DEFAULT 1 CHECK (role IN (1, 2, 3)),

    -- 1: active, 2: disabled, 3: pending, 4: locked
    status SMALLINT NOT NULL DEFAULT 3 CHECK (status IN (1, 2, 3, 4)),

    last_login_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now(),
    deleted_at TIMESTAMP -- Soft delete timestamp (NULL means active)
);

-- Partial Unique Indexes (only among non-deleted users)
CREATE UNIQUE INDEX idx_users_email ON users(email) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX idx_users_username ON users(username) WHERE deleted_at IS NULL AND username IS NOT NULL;

-- Only indexes the tiny percentage of users who are actually deleted
CREATE INDEX idx_users_deleted_users ON users(deleted_at) WHERE deleted_at IS NOT NULL;

-- Index for quickly filtering ghost vs real users
CREATE INDEX idx_users_ghost ON users(is_ghost) WHERE is_ghost = true;


-- Attach the auto-update trigger
SELECT trigger_updated_at('users');
```

---

## 2. Codebase Modifications

### A. Model Updates
- **[UserEntity](file:///home/schneider/Documents/socky-be/src/model/user/entity.rs)**:
  - Uncomment/add `username: Option<String>` and `full_name: Option<String>`.
- **User DTOs & VOs** (`src/model/user/dto.rs` / `vo.rs`):
  - Add `username` and `full_name` fields to relevant DTOs/VOs.
- **[AccessClaims](file:///home/schneider/Documents/socky-be/src/utils/token.rs)**:
  - Add `status: UserStatus` field. Update generation methods to include `UserStatus` when creating access tokens.

### B. Controller & Repository Updates
- **[AuthController::login](file:///home/schneider/Documents/socky-be/src/controller/auth_controller.rs#L93)**:
  - Do NOT reject `Disabled` users from logging in (bypass `check_status()` if status is `Disabled`, but keep blocking if `Pending` or `Locked`).
- **[AuthController::signup](file:///home/schneider/Documents/socky-be/src/controller/auth_controller.rs)**:
  - Add public signup method that handles hashing, validations, and creates user as `Pending`.
- **[user_repo.rs](file:///home/schneider/Documents/socky-be/src/repository/user_repo.rs)**:
  - Implement username lookup and existence checks (`username_exists`).
  - Add logic to find users by email/username.

### C. Middleware Updates
- **[auth_middleware](file:///home/schneider/Documents/socky-be/src/web/middleware/auth.rs#L18)**:
  - Read `status` from `AccessClaims`.
  - Validate state transitions and path restrictions as defined in [docs/reference/user_state_machine.md](file:///home/schneider/Documents/socky-be/docs/reference/user_state_machine.md).

---

## 3. API Endpoints

All endpoints below will be registered in [app.rs](file:///home/schneider/Documents/socky-be/src/app.rs).

### 3.0 Public Auth Endpoints (`/api/auth`)

| Endpoint | Action | Notes |
| :--- | :--- | :--- |
| `POST /api/auth/signup` | Registers a new user with `Pending` status. | Accepts `{ email, password, username, full_name }`. Requires admin/support reactivation. |

### 3.1 Administrative Namespace (`/api/user`)

Handles administrative control over users in the system.

| Endpoint | Action | Standard | Support | Admin |
| :--- | :--- | :---: | :---: | :---: |
| `GET /api/user` | Lists all users in the system. | Forbidden | Allowed | Allowed |
| `GET /api/user/{id}` | Retrieves a specific user's administrative details. | Forbidden | Allowed | Allowed |
| `PATCH /api/user/{id}` | Updates another user's profile information. | Forbidden | Allowed | Allowed |
| `POST /api/user/{id}/enable` | Enables the account if it was disabled, pending, or locked. | Forbidden | Allowed | Allowed |
| `POST /api/user/{id}/disable` | Disables account temporarily; the user can still reactivate it. | Forbidden | Allowed | Allowed |
| `POST /api/user/{id}/lock` | Locks account; the user can log in to delete, but can't do anything else. | Forbidden | Allowed | Allowed |
| `DELETE /api/user/{id}` | Deletes the user: anonymizes credentials, converts to ghost, revokes tokens. See [Ghost User Strategy](file:///home/schneider/Documents/socky-be/docs/reference/ghost_user_strategy.md). | Forbidden | Forbidden | Allowed |

### 3.2 Self-Service Namespace (`/api/user/me`)

Handles the currently logged-in user's personal account management.

| Endpoint | Action | Notes |
| :--- | :--- | :--- |
| `GET /api/user/me` | Fetches the currently authenticated user's full data. | |
| `PATCH /api/user/me/username` | Specific endpoint to update the unique handle. | Isolated for constraint validations. |
| `PATCH /api/user/me` | Updates general profile data (display name, etc.). | Payload of optional fields. |
| `POST /api/user/me/enable` | Enables account back if disabled. | Middleware must allow `Disabled` users here. |
| `POST /api/user/me/disable` | Disables account temporarily. | |
| `DELETE /api/user/me` | Deletes own account: anonymizes credentials, converts to ghost, revokes tokens. See [Ghost User Strategy](file:///home/schneider/Documents/socky-be/docs/reference/ghost_user_strategy.md). | Middleware must allow `Locked` users here. |

### 3.3 Social Discovery Namespace (`/api/profile`)

Handles the safe, read-only "Public Profile" pattern for friend and shared expense systems.

| Endpoint | Auth Required | Action | Notes |
| :--- | :---: | :--- | :--- |
| `GET /api/profile/check-username` | No (Public) | Verifies if a @username is available during registration. | Must be rate-limited (e.g., via IP) to prevent automated scraping or DoS attacks. |
| `GET /api/profile/search` | Yes (bearer-jwt) | Allows users to query for other users via their unique handle. | Protects against anonymous data scraping. |
| `GET /api/profile/{id}` | Yes (bearer-jwt) | Retrieves the public-facing details of a specific user. | Protects against anonymous data scraping. |
