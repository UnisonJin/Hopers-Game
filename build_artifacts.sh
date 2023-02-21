#!/bin/sh
for c in contracts/*; do
	pushd .
	cd ${c}
	cargo test
	if [[ $? -ne 0 ]]; then
		pwd
		echo Error: tests for the ${c} contract did not pass
		echo Refusing to build optimized wasms
		exit 1
	fi
	popd
done

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

pushd .
cd js
yarn
npm run build-schema
popd

WORKSPACE_OPTIMIZER_VERSION=0.12.8
if [[ `uname -m` == "arm64" ]]; then
  # M1 build
  docker run --rm -v "$(pwd)":/code \
    --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    cosmwasm/workspace-optimizer-arm64:${WORKSPACE_OPTIMIZER_VERSION}
  # Rename aarch64 wasms
  find artifacts -name '*-aarch64.wasm' -exec bash -c 'mv -f $0 ${0/-aarch64.wasm/.wasm}' {} \;
else
  docker run --rm -v "$(pwd)":/code \
    --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    cosmwasm/workspace-optimizer:${WORKSPACE_OPTIMIZER_VERSION}
fi

cargo fmt -- --check
if [[ $? -ne 0 ]]; then
	echo '*** Code was not linted with rustfmt ***'
	echo '*** Please run `cargo fmt` if you are planning to commit ***'
fi

