Leveraged Pool
==============

A pool providing leverage over a token. This contract is the main interface for
both liquidity providers and speculators minting and burning leveraged assets.

Ultimately two things are important in a leveraged pool: backing assets and
leveraged positions. The backing assets support leveraged positions such that
all positions are covered by some fraction of provided liqudity.

LPs provide the backing assets, while minters create leveraged assets by
providing one unit of the underlying in exchange for an equivalent amount of the
leveraged asset.

Architecture
------------

All entrypoints are in `src/contract.rs`. Any queries are routed to
`src/leverage_man.rs`. Liquidity positions are provided by `src/liquid_man.rs`
while leveraged positions are provided by `src/mint_man.rs`. Breaking out
functionality into different parts of the source helped facilitate parallel work
which was needed to deliver an MVP in the duration of Spacecamp.

`src/leverage_man.rs` ultimately tracks the backing asset's price history and
correlates that movement into the price of the leveraged asset.

Build
-----

You can build this module separately from the others in this repository though
it's seldom necessary:

```
cargo build
# Run unit tests
cargo test
```

License
-------

MIT License (available in the source tree as /LICENSE)

