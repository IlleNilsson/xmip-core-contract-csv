//! This contract's declaration: the contract guide,
//! `module/core/capability/contract/doc/adding-a-contract.md`.

use xcore::settings::Settings;

/// What a Location gives this contract (ADR-0064, amendment 2026-09-26):
/// nothing, since every CSV is held to its own header.
pub const SETTINGS: &Settings = &Settings::none(env!("CARGO_PKG_NAME"));

/// The technology's name, as a Validate step's `format` names it.
pub const NAME: &str = "csv";
