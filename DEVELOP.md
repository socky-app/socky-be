# Socky Backend Development

This document contains technical setup and development instructions for the Socky backend.

## Prerequisites

First of all, install a few usefull utilities.

```bash
cargo install sqlx-cli cargo-watch
```

Then configure the repository githooks.

```bash
git config --local core.hooksPath .githooks
```

Finally, create or local `.env` file and complete the missing values.

```bash
cp .env.example .env
```

> You migh want to change `SQLX_OFFLINE` to false if you are adding new queries.

## Starting the DB

Start postgresql server docker image:

```sh
docker compose up
```

Run the migrations:

```sh
sqlx migrate run
```

### First run

If this is the first time running the service, execute the `create_admin` script:

```sh
cargo run --bin create_admin
```

## Quick Dev

To watch both the server and the quick_dev in different terminals, execute the following:

```sh
# Terminal 1 - To run the server.
cargo watch -q -c -w src/ -x "run"

# Terminal 2 - To run the quick_dev.
cargo watch -q -c -w examples/ -x "run --example quick_dev"
```

> NOTE: Install cargo watch with `cargo install cargo-watch`.

## Unit Test

To watch all unit tests while developing, use the following command:

```sh
cargo watch -q -c -x "test -- --nocapture"
```

Instead, if you prefer to execute only one test:

```sh
# Specific test with filter.
cargo watch -q -c -x "test utils::hmac::tests::test_sha256_round_trip -- --nocapture"
```

## Implementation Roadmap

For the implementation plans, feature requirements, and technical phases, please refer to:

- [Implementation Roadmap](file:///home/schneider/Documents/socky-be/docs/plans/roadmap.md)
- [User State Machine Specifications](file:///home/schneider/Documents/socky-be/docs/reference/user_state_machine.md)
