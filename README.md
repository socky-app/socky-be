# Socky Backend

Backend repository for the Socky application

## Usage

TODO: Add usage instructions with conventional setup, using docker and docker compose

## Development

> NOTE: Install cargo watch with `cargo install cargo-watch`.

### Starting the DB

Start postgresql server docker image:

```sh
docker run --rm --name pg -p 5432:5432 \
   -e POSTGRES_PASSWORD=welcome \
   postgres:15
```

To have a psql terminal on pg, run psql on another terminal:

```sh
docker exec -it -u postgres pg psql
```

For pg to print all sql statements, run in psql:

```sh
ALTER DATABASE postgres SET log_statement = 'all';
```

### Quick Dev

To watch both the server and the quick_dev in different terminals, execute the following:

```sh
# Terminal 1 - To run the server.
cargo watch -q -c -w src/ -w .cargo/ -x "run"

# Terminal 2 - To run the quick_dev.
cargo watch -q -c -w examples/ -x "run --example quick_dev"
```

### Unit Test

To watch all unit tests while developing, use the following command:

```sh
cargo watch -q -c -x "test -- --nocapture"
```

Instead, if you prefer to execute only one test:

```sh
# Specific test with filter.
cargo watch -q -c -x "test model::task::tests::test_create -- --nocapture"
```
