#!/bin/bash

load_packages() {
  if [[ -d /src/pkgs ]]; then
    for pkg in /src/pkgs/core*.hart ; do
      hab pkg upload --url http://localhost:9636/v1 --auth "${HAB_AUTH_TOKEN}" "${pkg}" --channel stable
    done
  fi
}

load_package() {
  if [[ -d /src/pkgs ]]; then
    hab pkg upload --url http://localhost:9636/v1 --auth "${HAB_AUTH_TOKEN}" "$@" --channel stable
  fi
}

origin() {
  curl localhost:9636/v1/depot/origins \
    -d '{"name":"core"}' \
    -H "Authorization:Bearer:${HAB_AUTH_TOKEN}"
}

keys() {
  if [ -f ~/.hab/cache/keys/core-20160810182414.pub ]; then
    cat ~/.hab/cache/keys/core-20160810182414.pub | hab origin key import
  fi

  if [ -f ~/.hab/cache/keys/core-20160810182414.sig.key ]; then
    cat ~/.hab/cache/keys/core-20160810182414.sig.key | hab origin key import
  fi

  cat /hab/cache/keys/core-20160810182414.pub | \
  curl http://localhost:9636/v1/depot/origins/core/keys/20160810182414 \
    --data-binary @- \
    -H "Authorization:Bearer:${HAB_AUTH_TOKEN}"

  cat /hab/cache/keys/core-20160810182414.sig.key | \
  curl http://localhost:9636/v1/depot/origins/core/secret_keys/20160810182414 \
    --data-binary @- \
    -H "Authorization:Bearer:${HAB_AUTH_TOKEN}"
}

function psql() {
  hab pkg exec core/postgresql psql -U hab -h 127.0.0.1 "$@"
}

export -f load_packages
export -f origin
export -f keys
export -f psql
