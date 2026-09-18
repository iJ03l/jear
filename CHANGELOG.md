# Changelog

All notable changes to `jear` are documented here. Format follows Keep a Changelog, versioning follows SemVer.

## [Unreleased]

### Added

- `src/jev.rs`: `State`, `Choice`/`Score`/`Noul` Q+A types, `should_escalate()`, `clamp01()` + 3 tests.
- `src/policy.rs`: `Route`, `PolicyInput`, `Decision`, `decide()` budget x sensitivity engine + 5 tests.
- `src/near.rs`: `Hosting::{Tee,Anonymized,Proxied}` tiers + `privacy_rank()`/`meets_privacy()`, redacting `ApiKey` + 5 tests.
- `src/ironclaw.rs`: `Channel`, `DeployTarget`, `CostCaps`, `VaultRef` + 3 tests.
- `src/route.rs`: `Answers`, `route()`, `eligible_models()` + tier-aware `eligible_models_for()`, `within_caps()`, `route_with_monthly()`, `answers_from_wire()`, `estimate_plan()` + 8 tests.
- `src/budget.rs`: client-controlled `MonthlyBudget`, `remaining()`, `can_afford()`, `record()` + 3 tests.
- `src/jev_wire.rs`: SystemOne JSON `WireRequest`/`WireResponse` via `serde` + 3 tests.
- `src/near_wire.rs`: chat JSON + verbatim tools passthrough (`ChatTool`/`ToolCall`, `with_tools()`) + catalog `ModelEntry`/`ModelsResponse` with `estimated_cents()` + 6 tests.
- `src/http.rs`: bearer JSON `post_json()`/`get_json()`, `join()`, key-never-logged `HttpError` + 3 offline tests.
- `src/live.rs`: `evaluate()` + `complete()` + `list_models()` + `cheapest()`/`pick_best()` via env keys, caller-provided bases, offline tests + 6 tests.
- `src/attest.rs`: TEE `CpuQuote`/`AttestationReport` + nonce-bound `verify()` (full DCAP upstream) + 3 tests.
- `src/cli.rs`: `Config`, `parse()` brief + client `--monthly-cents` + `--live` + 4 tests.
- `src/lib.rs`: wires `attest`, `budget`, `cli`, `http`, `ironclaw`, `jev`, `jev_wire`, `live`, `near`, `near_wire`, `policy`, `route` modules.
- `src/main.rs`: CLI-driven — brief + monthly via `route_with_monthly()`; live Jev `evaluate()` → `answers_from_wire()` → route → NEAR `complete()` on catalog `pick_best()` priced by `estimate_plan()`, offline fallback.
- `tests/routing.rs`: 6 end-to-end loop tests (Jev JSON → route → monthly/TEE gates → catalog pick), offline, no assumptions.
- Deps: `serde` + `serde_json` + `ureq` with `json` feature (wire shapes + sync HTTP + live callers + CLI + live Jev loop + live catalog + attest done; server glue not built).
- Tooling: `rustfmt.toml`, `.github/workflows/ci.yml` (fmt + clippy + test + run).
- Docs: `README.md` quickstart + Docs index (`NEWBIE`, `KEYS`, `SERVER`); `docs/NEWBIE.md` 5-min start; `docs/KEYS.md` key sources + rotation + no-hosting answer; `docs/SERVER.md` per-tenant router-proxy design.

### Verified

- `cargo test --lib`: 53 passed; `cargo test --test routing`: 6 passed.
- `cargo run`: prints hello + `brief` + `monthly` + `route: DirectLlm` demo (8 lines).
- `cargo fmt --check`, `cargo clippy -- -D warnings`: clean.

## [0.1.0] - 2026-09-18

### Added

- Foundation crate: `Cargo.toml` (bin + lib, `MIT OR Apache-2.0`, edition 2021, Rust 1.70+).
- `src/lib.rs`: `VERSION`, `hello()` smoke API with unit test.
- `src/main.rs`: thin binary printing `hello()`.
- Open-source docs: `README.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `CHANGELOG.md`.
- Licenses: `LICENSE-MIT`, `LICENSE-APACHE`.
- Hygiene: `.gitignore` (target, `.env`, editors).

### Verified

- `cargo test --lib`: 1 passed.
- `cargo run`: prints `jear v0.1.0 — foundation ready`.
- Every file created one at a time and verified 3x (read-back + hash + build).

[0.1.0]: https://github.com/ij03l/jear/releases/tag/v0.1.0
