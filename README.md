# xmip-core-contract-csv

CSV contract: well-formedness is a parseable delimited file — consistent
field counts, balanced quotes — and conformance, when a layout is named, is
every row against that layout's columns and types (ADR-0042). A technology of
[xmip-core-contract](https://github.com/IlleNilsson/xmip-core-contract).

Declared and not yet written; `architecture.toml` carries the maturity.

## Toolchain

`rust-toolchain.toml` pins the toolchain for the whole estate. Do not change it
here.

## Verification

The included workflow is manual-only and calls the versioned shared workflow at
`IlleNilsson/.github@v1`.
