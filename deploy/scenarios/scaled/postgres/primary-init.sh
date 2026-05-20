#!/bin/bash
# Runs once, on first init of postgres-primary (docker-entrypoint-initdb.d).
# Creates the replication role and opens pg_hba so postgres-replica can stream
# WAL. Auth is scoped to the compose bridge network — never the host.
set -e

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-SQL
    CREATE ROLE ${POSTGRES_REPLICATION_USER:-replicator}
        WITH REPLICATION LOGIN PASSWORD '${POSTGRES_REPLICATION_PASSWORD:-replicator}';
SQL

# Allow replication connections from any host on the container network.
# The bridge network is not reachable from the host, so this is contained.
{
    echo "host replication ${POSTGRES_REPLICATION_USER:-replicator} all md5"
    echo "host all          ${POSTGRES_REPLICATION_USER:-replicator} all md5"
} >> "$PGDATA/pg_hba.conf"
