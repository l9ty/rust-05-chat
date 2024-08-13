#!/bin/sh

VOLUME_NAME=dev-postgres-v
CONTAINER_NAME=dev-postgres-c

docker volume inspect $VOLUME_NAME >/dev/null 2>&1 || docker volume create $VOLUME_NAME
docker inspect >/dev/null 2>&1 || docker run -itd --name $CONTAINER_NAME \
    -p 5432:5432 \
    -e POSTGRES_PASSWORD=postgres \
    -e POSTGRES_DB=chat \
    -v $VOLUME_NAME:/var/lib/postgresql/data \
    postgres:16
