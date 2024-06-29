# mailomat

My version of the app built in [Zero To Production In Rust](https://www.zero2prod.com). 
Uses Axum instead of Actix.

## Use: 

Before running you need: 
    - Docker
    - TODO

Development: 
    - It needs a running Postgres database for the tests to work. 
    You can initialize the Docker image running a PG_DB with `scripts/init_docker_db.sh` script.
    By default the script tries to delete the previous Docker image with the same name, 
    if running for the first time you need to run it like so:
        - BASH: `SKIP_DB_RESET=1 ./scripts/init_docker_db.sh` 
        - FISH: `env SKIP_DB_RESET=1 ./scripts/init_docker_db.sh`
