# Ghost User Strategy

This document defines how the Socky application handles non-registered and deleted users in a unified way.

---

## 1. Problem

Financial records (expenses, splits, settlements) reference users via foreign keys. When a user is deleted or was never registered, those references must remain valid and meaningful to other users in the system.

## 2. Design Principle

From the perspective of any other user, there is a single concept: **a ghost user** — a person who exists in financial history but is not an active participant. Whether that person never signed up, or deleted their account is irrelevant to other users.

## 3. Ghost User Definition

A ghost user is a row in the `users` table with:

- `is_ghost = true`
- `full_name` preserved (the only visible identity)
- `email`, `password_hash`, `username` set to `NULL`
- No ability to log in or perform any action

## 4. How Users Become Ghosts

| Scenario | Trigger | Result |
| :--- | :--- | :--- |
| **Never registered** | Added to a group split by name | Row created with `is_ghost = true`, only `full_name` set |
| **Self-delete** | `DELETE /api/user/me` | Anonymized and converted to ghost |
| **Admin delete** | `DELETE /api/user/{id}` | Anonymized and converted to ghost |

## 5. Deletion Process

When a real user is deleted, the following happens atomically:

### A. Credential Anonymization

1. `email` → `NULL`
2. `password_hash` → `NULL`
3. `username` → `NULL`
4. `full_name` → preserved (so other users still see a recognizable name)
5. `is_ghost` → `true`
6. `deleted_at` → `now()`

### B. Session Cleanup

7. All active refresh tokens are revoked

> [!IMPORTANT]
> Deletion is **final**. There is no grace period or account recovery. Users who want a reversible option should use the **Disable** feature instead.

## 6. Orphan Data Cleanup (Background Worker)

Data cleanup is **not** performed at delete time. The delete endpoint is kept fast and simple (anonymize + ghost + revoke tokens). A background worker handles cleanup separately.

### Why a Worker Instead of Eager Cleanup

Financial records form a graph of relationships between users. When one user deletes their account, their shared records must remain for other participants. Only when **all** participants in a record are ghosts can it be safely removed. Eager cleanup at delete time would require scanning the full relationship graph, which is complex and error-prone. A periodic worker handles cascading cleanup naturally.

### Worker Logic

The worker periodically scans for orphaned financial data:

1. **Find ghost-only expenses**: expenses where every participant (`payer_id` and all `expense_split.user_id`) is a ghost → delete the expense and its splits
2. **Find ghost-only groups**: groups where every member is a ghost → delete the group, all its expenses, and related records
3. **Find ghost-only settlements**: settlements where both `sender_id` and `receiver_id` are ghosts → delete

After cleanup, ghost user rows that no longer have any financial records referencing them may optionally be hard-deleted (physically removed from the database).

> [!NOTE]
> The cleanup worker is deferred to the phase where financial tables are implemented (Phase 2+). Until expenses and groups exist, there is nothing to clean up.

## 7. Foreign Key Strategy for Financial Tables

All financial tables must use `ON DELETE RESTRICT` on user foreign keys:

```sql
payer_id BIGINT NOT NULL REFERENCES users(id) ON DELETE RESTRICT
user_id  BIGINT NOT NULL REFERENCES users(id) ON DELETE RESTRICT
```

This ensures the ghost row cannot be accidentally removed while shared financial records still reference it.

## 8. Querying Ghost Users

When displaying financial records to other users:
- If `is_ghost = false`: show full profile (name, username, avatar link)
- If `is_ghost = true`: show only `full_name` with a "ghost" indicator, no profile link

## 9. Ghost User Merging (Future)

When a ghost user (created by name for splitting) later signs up with a real account, their ghost record can be **merged** into the new account. This transfers all financial history to the real user's `id`. This feature is deferred to a future phase.

## 10. Schema Change

Add the following column to the `users` table:

```sql
is_ghost BOOLEAN NOT NULL DEFAULT false
```
