#!/usr/bin/env bash
set -eo pipefail

RUNNING_CONTAINER=$(docker ps --filter 'name=mailomat-redis' --format '{{.ID}}')
if [[ -n $RUNNING_CONTAINER ]]; then
  echo >&2 "There is a Redis container already running!"
  echo >&2 "Kill it with: 'docker kill ${RUNNING_CONTAINER}'"
  exit 1
fi

if [[ -z "${SKIP_DB_RESET}" ]]; then
  docker rm mailomat-redis
  echo >&2 " — Removed existing container named 'mailomat-redis'!"
fi
# launch redis using docker
docker run -p "6379:6379" \
  -d \
  --name "mailomat-redis" \
  redis:8

>&2 echo " ———>    Redis is ready to go!"
