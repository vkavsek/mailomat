#!/usr/bin/env bash
set -eo pipefail

if ! [ -x "$(command -v psql)" ]; then 
	echo >&2 "Error: psql is not installed."
	exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then 
	echo >&2 "Error: sqlx is not installed."
	echo >&2 "Use:"
	echo >&2 "	cargo install --version='~0.7' sqlx-cli \
		--no-default-features --features rustls,postgres"
	echo >&2 "to install it."

	exit 1
fi

# Check if a custom user has been set, otherwise default to 'postgres'
DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=newsletter}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_HOST="${POSTGRES_HOST:=localhost}"

if [[ -z "${SKIP_DB_RESET}" ]]
then
	docker rm mailer_pg
	echo >&2 " — Removed existing container named 'mailer_pg'!"
fi
if [[ -z "${SKIP_DOCKER}" ]]
then
	docker run -d --name mailer_pg \
		-e POSTGRES_USER=${DB_USER} \
		-e POSTGRES_PASSWORD=${DB_PASSWORD} \
		-e POSTGRES_DB=${DB_NAME} \
		-p "${DB_PORT}":5432 \
		postgres:16 \
		postgres -N 1000
		# ^ Increased maximum number of connections for testing purpouses
	echo >&2 " — Started a new Docker container called 'mailer_pg'!"
fi
	
# Try to run a psql command to check if DB is online.
export PGPASSWORD="${DB_PASSWORD}"
until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres"  -c '\q'; do 
	echo >&2 " — Postgres is still not available — Sleeping."
	sleep 1
done

echo >&2 " — Postgres is up and running on port ${DB_PORT}!"

DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
export DATABASE_URL
sqlx database create
sqlx migrate run

echo >&2 " — Postgres has been migrated, ready to go!"
