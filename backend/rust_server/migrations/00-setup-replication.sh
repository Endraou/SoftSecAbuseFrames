#!/bin/bash
set -e

# 1. Autoriser explicitement notes_admin pour la réplication miaou
echo "host replication notes_admin 0.0.0.0/0 trust" >> "$PGDATA/pg_hba.conf"

# 2. S'assurer que l'utilisateur a bien le flag REPLICATION en base miaou
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    ALTER USER notes_admin WITH REPLICATION;
EOSQL