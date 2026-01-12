#!/bin/bash
set -e

# Add replication permissions to pg_hba.conf
echo "host replication all 0.0.0.0/0 trust" >> "$PGDATA/pg_hba.conf"