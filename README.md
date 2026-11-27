# xmip-core-contract-csv

CSV contract: well-formedness is a parseable delimited file — consistent
field counts, balanced quotes — and conformance, when a layout is named, is
every row against that layout's columns and types (ADR-0042). A technology of
[xmip-core-contract](https://github.com/IlleNilsson/xmip-core-contract).

Declared and not yet written; `architecture.toml` carries the maturity.

## The declaration

What a Location naming this contract is configured with is two `const`s in
its `declaration` module, `src/declaration.rs`, beside the code, which its
`ContractFactory` reads (ADR-0064, amendment 2026-11-26):
`declaration::SETTINGS`, none, and `NAME`, `csv`, the name a Validate
step's `format` names it by. Built with its `declaration` feature alone
(`--no-default-features --features declaration`), the crate is those and
nothing more: its code and every crate the code is built on are behind the
default feature. So the library the surfaces load can carry it without the
code, for the language server and the desktop to suggest and check a
Location's contract settings though no node in their process loaded the
technology, once the gathering crate of open problem 48 links it.
`test/Declaration.Test.ps1` in the estate holds that the declaration builds
nothing the contract capability does not.

## Verification

The included workflow is manual-only and calls the versioned shared workflow at
`IlleNilsson/.github@v1`.
