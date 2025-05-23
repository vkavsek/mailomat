# mailomat - WIP

My version of the app built in [Zero To Production In Rust](https://www.zero2prod.com).
Uses [Axum](https://github.com/tokio-rs/axum) instead of Actix with hosting on [Fly.io](https://fly.io/).

[TODOs](TODO.md)

## Info

### Before running you need

- [mold](https://github.com/rui314/mold)
- [Docker](https://www.docker.com/)
- [sqlx](https://github.com/launchbadge/sqlx) CLI app
- [psql](https://www.postgresql.org/download/)
- [flyctl](https://fly.io/docs/flyctl/install/) (for Deployment and Monitoring only)

### Development

- It needs a running Postgres & Redis databases for the tests to work.
- You can initialize the Docker images running Postgres and Redis databases with `./scripts/init_databases.sh` script.

The script first tries to do a reset by deleting the image left behind from the previous day.
If running on a system for the first time you need to run it like so:

```sh
SKIP_DB_RESET=1 ./scripts/init_databases.sh
```

```fish
env SKIP_DB_RESET=1 ./scripts/init_databases.sh
```

### Testing

You can control the level of logs emitted by test with `TEST_LOG` enviroment variable.
It works the same way as `RUST_LOG`.

#### Config notes

Currently [figment](https://github.com/SergioBenitez/Figment) is used to build the config. You can inject values at runtime with enviroment variables
that start with a prefix `CONFIG__`, and fields separated by `__` like so:

```sh
CONFIG__NET_CONFIG__APP_PORT=8000
```

#### SQLX notes

[SQLX](https://github.com/launchbadge/sqlx) uses .env file to access **DATABASE_URL** enviroment variable for static checking (`query` macro).

#### Tera notes

[Tera](https://keats.github.io/tera/) is used as a templating engine. You can modify the templates in the templates folder.

#### Redis notes

- Redis is used for session management because [fly.io](https://fly.io/docs/upstash/redis/) offers a managed Redis database service via [Upstash](https://upstash.com/docs/redis/overall/getstarted)
- It could easily be swapped out for Valkey which can be deployed on fly.io with [flycast](https://fly.io/docs/blueprints/private-applications-flycast/).
  Valkey seems more trustworthy, but the upstash-redis provides high availability, seems pretty cheap and probably handles backups etc.

### Deployment

CI deploys to Fly.io automatically if all the checks are succesful.
If you need to deploy locally you can do so with:

```fish
fly deploy --local-only
```

### Database Management

#### Dev

You can connect to a PGDB running in a Docker container using `psql`.
Check out the **init_docker_db.sh** for syntax.

Common PSQL commands:

- `\l` : list all databases
- `\c {dbname}` : connect to a database
- `\d` : list all tables in the database
- `\d {table_name}` : list the structure of a table

#### Production

To connect to and monitor the DB:

```fish
fly pg connect -a mailomat-pg
```

To migrate the database using `sqlx` (reset / update):

1. Forward the server port to your local system:

```fish
fly proxy 15432:5432 -a mailomat-pg
```

2. Then you can use `DATABASE_URL` env variable to migrate, don't forget to replace the password:

```fish
env DATABASE_URL="postgres://postgres:<password>@localhost:15432/mailomat" sqlx migrate run
```
