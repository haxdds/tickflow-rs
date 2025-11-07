# This command starts a new Docker container running PostgreSQL.
# Syntax breakdown:
# - `docker run`: Start a new container from an image.
# - `--name postgres`: Name the container "postgres" for easier references.
# - `-e POSTGRES_PASSWORD=password`: Set the environment variable POSTGRES_PASSWORD to "password" (PostgreSQL will use this as the database password).
# - `-p 5432:5432`: Map port 5432 on your machine to port 5432 on the container (so you can connect from your host).
# - `-d`: Run the container in "detached" mode (in the background).
# - `postgres`: The Docker image to use (the official PostgreSQL image from Docker Hub).

docker run --name postgres -e POSTGRES_PASSWORD=password -p 5432:5432 -d postgres

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
