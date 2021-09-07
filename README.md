Leveraged Tokens on Terra network
=================================

Concept for zero-sum leveraged trading pools on Terra network. Created for
Terra's Spacecamp hackathon.

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

Contracts created from this repository are available on the bombay-10 test
network at the below addresses:

| Contract Name                                      | Address                                  |
| -------------------------------------------------- | -------------------------------------------- |
| Factory                                            | [terra16sjkmp79wku8hg3su7uqxqgm9770r0ddz8kdq8](https://finder.terra.money/bombay-10/address/terra16sjkmp79wku8hg3su7uqxqgm9770r0ddz8kdq8)|
| mTSLA 2x leverage pool                             | [terra14kqxdu9rv97dhrwq3ns444rnwhzz0s72k0nt7d](https://finder.terra.money/bombay-10/address/terra14kqxdu9rv97dhrwq3ns444rnwhzz0s72k0nt7d)|
| MIR 3x leverage pool                               | [terra1kn9e6pmcqsynkmu4vra4wmxv0f4f5m356ul6re](https://finder.terra.money/bombay-10/address/terra1kn9e6pmcqsynkmu4vra4wmxv0f4f5m356ul6re)|

Minting a leveraged position requires access to the mocked
[MIR](https://finder.terra.money/bombay-10/address/terra1k59qq3pxj93arv399l4a90ndewn50gfy8nkcn2)
or
[mTSLA](https://finder.terra.money/bombay-10/address/terra1dsh6lll9av4dqk57juavk6dg4yzh9twhe600z6)
assets which back the leveraged positions.

Building
--------

Building is facilitated by a shell script in this repository
(`./build_artifacts.sh`) which uses the `cosmwasm/workspace-optimizer` Docker
container to build artifacts that are optimized for deployment on Terra network.
It also builds all schemas and enforces the rule that all unit-tests must pass.

Here are the steps for building artifacts from scratch:

```
git clone https://github.com/SpaceCamp-Leverage-Tokens/leveraged_pools
cd leveraged_pools
cargo build --all
./build_artifacts.sh
```

License
-------

All code is under the very permissive MIT License (available in the source tree
as /LICENSE) so pretty much do whatever you want.

