#!/bin/sh

set -euxo pipefail

cargo update -vw
[[ -z "$(git status --porcelain)" ]]


for i in $(seq 10)
do
    cargo publish && exit 0
    sleep 5
done
cargo publish
