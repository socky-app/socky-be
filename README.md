# Socky Backend

Backend repository for the Socky application

## Usage

TODO: Add usage instructions with conventional setup, using docker and docker compose

## Development

First of all, configure the githooks.

```bash
git config --local core.hooksPath .githooks
```

### Starting the DB

Start postgresql server docker image:

```sh
docker compose up
```

Run the migrations:

```sh
sqlx migrate run
```

#### First run

If this is the first time running the service, execute the `create_admin` script:

```sh
cargo run --bin create_admin
```

### Quick Dev

To watch both the server and the quick_dev in different terminals, execute the following:

```sh
# Terminal 1 - To run the server.
cargo watch -q -c -w src/ -x "run"

# Terminal 2 - To run the quick_dev.
cargo watch -q -c -w examples/ -x "run --example quick_dev"
```

> NOTE: Install cargo watch with `cargo install cargo-watch`.

### Unit Test

To watch all unit tests while developing, use the following command:

```sh
cargo watch -q -c -x "test -- --nocapture"
```

Instead, if you prefer to execute only one test:

```sh
# Specific test with filter.
cargo watch -q -c -x "test utils::hmac::tests::test_sha256_round_trip -- --nocapture"
```

## Nest steps

- Auth
  - Define sign-up token logic
  - Create invite endpoint
  - Create sign-up endpoint
- User
  - CRUD (behind admin only)
