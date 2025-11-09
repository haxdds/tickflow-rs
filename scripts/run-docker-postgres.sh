# This command starts a new Docker container running PostgreSQL for local development.

docker run --name tickflow-postgres -e POSTGRES_PASSWORD=password -p 5432:5432 -d postgres

# How to connect to this database from outside (e.g., from psql or a client library):
# 
# Host: localhost
# Port: 5432
# User: postgres
# Password: password
# Database: postgres (default)
#
# Example psql connect string:
#   psql -h localhost -U postgres -d postgres
#
# Example PostgreSQL URI:
#   postgres://postgres:password@localhost:5432/postgres
