#!/bin/sh
for c in contracts/*; do
    pushd .
    cd ${c}
    cargo schema
    if [[ $? -ne 0 ]]; then
        pwd
        echo Error: schemas for the ${c} contract did not build
        echo Refusing to build optimized wasms
        exit 1
    fi
    popd
done

(
cd js
yarn
npm run build-schema
)

