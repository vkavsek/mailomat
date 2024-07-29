# mailomat
### WIP

My version of the app built in [Zero To Production In Rust](https://www.zero2prod.com). 
Uses Axum instead of Actix with hosting on Fly.io.

## Info: 

### Before running you need: 
- Docker
- sqlx
- psql
- flyctl (for Deployment and Monitoring only)

### Development: 
- It needs a running Postgres database for the tests to work. 
You can initialize the Docker image running a PG_DB with `scripts/init_docker_db.sh` script.
By default the script tries to delete the previous Docker image with the same name, 
if running for the first time you need to run it like so:
#### bash
```sh
SKIP_DB_RESET=1 ./scripts/init_docker_db.sh
```
#### fish
```fish
env SKIP_DB_RESET=1 ./scripts/init_docker_db.sh
```

### Deployment: 
CI deploys to Fly.io automatically if all the checks are succesful.
If you need to deploy locally you can do so with:
```fish
fly deploy --local-only
```

### Database Management: 
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
