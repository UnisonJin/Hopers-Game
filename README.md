Førecast Prediction Markets
===========================

Prediction markets let users bet on the short time direction of a desired ticker

Contracts
---------

The source code for each contract is in the [`contracts/`](contracts/)
directory.

| Name                                               | Description                            |
| -------------------------------------------------- | -------------------------------------- |
| [`price-prediction`](contracts/price_prediction) | Central place managing positions and reward distribution |
| [`fast-oracle`](contracts/fast_oracle)       | Oracle for on-chain data |

Build + Optimize Wasm Artifacts
-------------------------------

Compiling and optimizing requires these tools to be installed:

+ Rust compiler (1.63.0)
+ wasm-opt (version 105)

Further if you'd like to compile schemas for the contracts you need to have:

+ yarn

If using the nix package manager just run `nix-shell` to be loaded into an
environment with the correct versions of these tools installed.

```
./optimize.sh
```

On-chain Blob Verification
------------------------

You can validate that this source code produced the blob that is powering the
on-chain contract. Make sure you have `junod` and all the dependencies above
installed then simply run these two scripts:

```
./optimize.sh
./validate_onchain_wasm.sh
```

Testing + Schemas
-----------------

Contract behavior verified with cw-multitest. Run tests with:

```
cargo test
```

Schemas are built using cosmwasm-schema. To build, run:

```
./build_schema.sh
```

Schemas will appear in `contracts/${c}/schema` and TS definitions are written to
`contracts/${c}/ts`.

License
-------

MIT License (see `/LICENSE`)
