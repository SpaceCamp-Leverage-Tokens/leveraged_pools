#!/bin/sh

cargo unit-test
if [[ $? -ne 0 ]]; then
	echo ERROR: Cargo tests did not pass
	echo Refusing to build optimized wasms
	exit 1
fi

for c in contracts/*; do
	pushd .
	cd ${c}

	cargo schema
	if [[ $? -ne 0 ]]; then
		pwd
		echo Error: schemas for the ${c} contract did not build correctly
		echo Refusing to build optimized wasms
		exit 1
	fi
	popd
done

docker run --rm -v "$(pwd)":/code \
	--mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
	--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
	cosmwasm/workspace-optimizer:0.11.4

cargo fmt -- --check
if [[ $? -ne 0 ]]; then
	echo '*** Code was not linted with rustfmt ***'
	echo '*** Please run `cargo fmt` if you are planning to commit ***'
	exit 1
fi

