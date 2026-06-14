# Phase 2: Core Expense & Income

> [!NOTE]
> This plan is not yet structured. It collects items identified during Phase 1 planning that are deferred to Phase 2, alongside the original roadmap goals. A detailed implementation plan will be created when Phase 2 work begins.

---

## Original Roadmap Goal

Implement personal transaction tracking (debit vs. credit card cycles with installment dates) and income tracking (without splits).

---

## Deferred Items from Phase 1

### Ghost User Orphan Cleanup Worker

When users are deleted in Phase 1, the delete endpoint only anonymizes credentials, converts the user to a ghost, and revokes tokens. No financial data cleanup is performed at delete time.

A **background worker** must be implemented to periodically scan for and clean up orphaned financial data:

1. **Ghost-only expenses**: expenses where every participant (`payer_id` and all `expense_split.user_id`) is a ghost → delete the expense and its splits
2. **Ghost-only groups**: groups where every member is a ghost → delete the group, all its expenses, and related records
3. **Ghost-only settlements**: settlements where both `sender_id` and `receiver_id` are ghosts → delete

After cleanup, ghost user rows with zero remaining financial references may optionally be hard-deleted (physically removed from the database).

See [Ghost User Strategy](file:///home/schneider/Documents/socky-be/docs/reference/ghost_user_strategy.md) for full context.

### Ghost User Merging

When a ghost user (created by name for expense splitting with a non-registered person) later signs up with a real account, their ghost record should be **merged** into the new account, transferring all financial history to the real user's `id`.

### Foreign Key Strategy

All financial tables must use `ON DELETE RESTRICT` on user foreign keys to prevent accidental removal of ghost rows that are still referenced by shared records. See [Ghost User Strategy §7](file:///home/schneider/Documents/socky-be/docs/reference/ghost_user_strategy.md) for the FK design.
