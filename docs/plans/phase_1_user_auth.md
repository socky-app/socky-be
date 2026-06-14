# Phase 1: User Profile Extensions & Authorization Logic

This document specifies the database migrations, codebase changes, and API endpoints required to implement Phase 1.

---

## 1. Database Migrations

Create a new migration file `xxxx_add_user_profile_columns.up.sql` to extend the `users` table:
```sql
-- Up Migration
ALTER TABLE users 
ADD COLUMN username TEXT COLLATE "case_insensitive" UNIQUE,
ADD COLUMN full_name TEXT;

-- Create partial unique index to allow username to be null or unique when not deleted
CREATE UNIQUE INDEX idx_users_username ON users(username) WHERE deleted_at IS NULL AND username IS NOT NULL;
```

And corresponding down migration:
```sql
-- Down Migration
DROP INDEX IF EXISTS idx_users_username;
ALTER TABLE users 
DROP COLUMN IF EXISTS username,
DROP COLUMN IF EXISTS full_name;
```

---

## 2. Codebase Modifications

### A. Model Updates
- **[UserEntity](file:///home/schneider/Documents/socky-be/src/model/user/entity.rs)**:
  - Uncomment/add `username: Option<String>` and `full_name: Option<String>`.
- **User DTOs & VOs** (`src/model/user/dto.rs` / `vo.rs`):
  - Add `username` and `full_name` fields.
- **[AccessClaims](file:///home/schneider/Documents/socky-be/src/utils/token.rs)**:
  - Add `status: UserStatus` field. Update generation methods to read `UserStatus` when creating access tokens.

### B. Controller & Repository Updates
- **[AuthController::login](file:///home/schneider/Documents/socky-be/src/controller/auth_controller.rs#L93)**:
  - Do NOT reject `Disabled` users from logging in (bypass `check_status()` if status is `Disabled`, but keep blocking if `Pending` or `Locked`).
- **[user_repo.rs](file:///home/schneider/Documents/socky-be/src/repository/user_repo.rs)**:
  - Implement username lookup and existence checks (`username_exists`).

### C. Middleware Updates
- **[auth_middleware](file:///home/schneider/Documents/socky-be/src/web/middleware/auth.rs#L18)**:
  - Read `status` from `AccessClaims`.
  - Validate state transitions and path restrictions as defined in [docs/reference/user_state_machine.md](file:///home/schneider/Documents/socky-be/docs/reference/user_state_machine.md).

---

## 3. API Endpoints

We will implement the following endpoints and register them in [app.rs](file:///home/schneider/Documents/socky-be/src/app.rs):

### User Self-Management (`/api/user/me`)
- `GET /api/user/me`: Returns the currently authenticated user's profile information.
- `POST /api/user/me/disable`: Self-deactivates the user's status to `Disabled`.
- `POST /api/user/me/enable`: Self-activates the user's status back to `Active` (accessible to `Disabled` users).
- `PATCH /api/user/me`: Updates profile details (like display name, full name, avatar, etc.).

### Profiles (`/api/profile`)
- `GET /api/profile/check-username?username=...`: Publicly queries if a username handle is available.
- `GET /api/profile/search?query=...`: Searches for users via their unique username handle (requires bearer authentication).
