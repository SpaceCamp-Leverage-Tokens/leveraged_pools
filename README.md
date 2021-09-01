Leveraged Tokens on Terra network
=================================

Concept for zero-sum leveraged trading pools on Terra network. Created for
Terra's Spacecamp hackathon.

Building
--------

To build all artifacts run this build script:

```
./build_artifacts.sh
```

The script will build all schemas and run unit tests as well; if any of those
steps fail artifacts will not be built. If the script doesn't run on your
platform then artifacts can be built with these commands run from this
directory:

```
# Optionally run all unit tests
# cargo unit-test
cargo build wasm
docker run --rm -v "$(pwd)":/code \
	--mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
	--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
	cosmwasm/workspace-optimizer:0.11.4
```

Contracts
---------

This concept relies on three contracts; the source code is in the
[`contracts/`](contracts/) directory; each contract has a README roughly
explaining its contribution to the leveraged token system.

| Name                                               | Description                                  |
| -------------------------------------------------- | -------------------------------------------- |
| [`leveraged_pool`](contracts/leveraged_pool)       | Pool providing leverage on a token           |
| [`factory`](contracts/factory)                     | Create leveraged pools
| `leverage_governance`                              | WIP                                          |


License
-------

All code is under the very permissive MIT License (available in the source tree
as /LICENSE) so pretty much do whatever you want.

