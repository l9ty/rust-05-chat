#!/bin/sh

VOLUME_NAME=dev_postgres

docker volume inspect $VOLUME_NAME >/dev/null 2>&1 || docker volume create $VOLUME_NAME
docker run -itd --name postgres \
    -p 5432:5432 \
    -e POSTGRES_PASSWORD=postgres \
    -e POSTGRES_DB=chat-dev \
    -v $VOLUME_NAME:/var/lib/postgresql/data \
    postgres:16
