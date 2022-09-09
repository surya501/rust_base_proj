#!/usr/bin/env bash
# Run this script like this:
# $ SKIP_DOCKER=true ./scripts/init_db.sh

set -x
set -eo pipefail

#check if psql is installed
if ! [ -x "$(command -v psql)" ]; then
  echo 'Error: psql is not installed.' >&2
  exit 1
fi

# check if sqlx is installed
if ! [ -x "$(command -v sqlx)" ]; then
  echo 'Error: sqlx is not installed.' >&2
  exit 1
fi


#Check if custom user has been set, otherwise use default
DB_USER=${POSTGRES_USER:=postgres}
DB_NAME=${POSTGRES_DB:=newsletter}
DB_PASS=${POSTGRES_PASSWORD:=postgres}
DB_PORT=${POSTGRES_PORT:=5432}

# Kill the existing docker image if it exists
docker kill newsletter-db || true
if [[ -z "$SKIP_DOCKER" ]]; then
  # Launch postgres using Docker
  docker run --rm --name newsletter-db -e POSTGRES_USER=$DB_USER -e POSTGRES_PASSWORD=$DB_PASS -e POSTGRES_DB=$DB_NAME -p $DB_PORT:5432 -d postgres postgres -N 1000
  # docker run -e POSTGRES_USER=$DB_USER -e POSTGRES_PASSWORD=$DB_PASS -e POSTGRES_DB=$DB_NAME -p $DB_PORT:5432 -d postgres postgres -N 1000
  # Wait for the database to start
  sleep 5
fi

# keep pinging postgres until it's ready to accept commands
export PGPASSWORD=$DB_PASS
until psql -h localhost -U $DB_USER -p $DB_PORT -d $DB_NAME -c '\q'; do
  >&2 echo "Postgres is unavailable - sleeping"
  sleep 1
done

>&2 echo "Postgres is up - executing command"

export DATABASE_URL=postgres://$DB_USER:$DB_PASS@localhost:$DB_PORT/$DB_NAME
sqlx database create
sqlx migrate run

>&2 echo "Postgres has been migrated, ready to go!"