# Phase 0: Source Code Documentation Baseline

This document describes the tasks and standards to establish a comprehensive documentation baseline for the Socky backend.

---

## 1. Goal

Ensure that every module, public struct, enum, and function in the application has high-quality Rust doc comments (`///` and `//!`). Running `cargo doc` should generate fully navigable, descriptive API reference documentation without any compilation warnings.

---

## 2. Scope of Work

We will enrich documentation across the codebase modules:

### A. Root & App Structure
- **[lib.rs](file:///home/schneider/Documents/socky-be/src/lib.rs)**: Expand top-level architecture outline detailing layout modules (`app`, `context`, `controller`, `model`, `repository`, `utils`, `web`).
- **[main.rs](file:///home/schneider/Documents/socky-be/src/main.rs)**: Document initialization logic, environment parsing, and runtime worker spawning.
- **[src/app/state.rs](file:///home/schneider/Documents/socky-be/src/app/state.rs)**: Document state variables stored across HTTP handler requests (`AppState`).

### B. Core Utilities
- **[src/utils/hmac.rs](file:///home/schneider/Documents/socky-be/src/utils/hmac.rs)**: Document hashing algorithms, salt/pepper usage, and encryption utilities.
- **[src/utils/password.rs](file:///home/schneider/Documents/socky-be/src/utils/password.rs)**: Document password verification and Argon2 PEPPER config hashing.
- **[src/utils/token.rs](file:///home/schneider/Documents/socky-be/src/utils/token.rs)**: Document JWT encoding, decoding, Opaque token generation, and claims mappings.

### C. Web & Middleware Layer
- **[src/web/middleware/auth.rs](file:///home/schneider/Documents/socky-be/src/web/middleware/auth.rs)**: Document Bearer extraction and claims injection.
- **[src/web/middleware/core.rs](file:///home/schneider/Documents/socky-be/src/web/middleware/core.rs)**: Document tracing headers, request-id assignment, and panic catching.
- **[src/web/extractor/require_role.rs](file:///home/schneider/Documents/socky-be/src/web/extractor/require_role.rs)**: Document RBAC extractor functionality.

### D. Repository & Database Operations
- **[src/repository/ops.rs](file:///home/schneider/Documents/socky-be/src/repository/ops.rs)**: Document the generic CRUD design (`Create`, `Get`, `Update`, `Delete`, `SoftDelete` traits).
- **[src/repository/ops/delete_strategy.rs](file:///home/schneider/Documents/socky-be/src/repository/ops/delete_strategy.rs)**: Explain standard hard vs soft delete behaviors.
- **[src/repository/auth_repo.rs](file:///home/schneider/Documents/socky-be/src/repository/auth_repo.rs)**: Document token rotation, query locking, and refresh token validation queries.
- **[src/repository/user_repo.rs](file:///home/schneider/Documents/socky-be/src/repository/user_repo.rs)**: Document user entity queries.

---

## 3. Success Criteria & Validation

To verify that the documentation baseline is correctly established:
1. Run the following command:
   ```bash
   cargo doc --no-deps --document-private-items
   ```
2. The compilation must exit successfully with `0` warnings.
3. Open the generated documentation pages to verify navigation structure, description clarity, and layout design.
