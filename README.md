# jear

Jev-routed client for NEAR AI Cloud inference and IronClaw agents.

Users never pick a model or agent. `jear` routes by **budget**, **quality**, and **sensitivity** using Jev (TypeSafe AI System One) for fast, structured decisions.

- Status: `v0.1.0` core routing + wire JSON + HTTP helper — demo offline, live via env keys.
- Stack: Rust 1.70+ (tested on 1.96.0), `serde`/`serde_json`/`ureq`.
- License: `MIT OR Apache-2.0` — free for everyone.

## Why

- Jev decides: `Choice` (which agent), `Score` (complexity/sensitivity), `Noul` (needs private TEE? risky? urgent?) with calibrated confidence.
- NEAR AI Cloud executes: OpenAI-compatible `https://cloud-api.near.ai/v1`, TEE-hosted private models with attestation, plus proxied frontier models.
- IronClaw acts: secure Rust agent OS, personal instances + Agent Marketplace hires.

## Quick start

```bash
cargo test
cargo run -- "brief in plain English" --monthly-cents 5000
```

Expected:

```text
jear v0.1.0 — foundation ready
brief: Summarize my last 3 payout-failure emails
monthly: 5000c remaining
route: DirectLlm (tee: false)
reason: simple lookup — direct inference
eligible models: 2
caps ok for 5c: true
target: Hub("demo-instance")
```

Live (needs key, else stays offline with guidance):

```bash
export NEAR_API_KEY="sk-..."
cargo run -- "brief in plain English" --monthly-cents 5000 --live
```

## Project layout

```text
jear/
├── Cargo.toml
├── src/
│   ├── lib.rs       # library root + module wiring
│   ├── main.rs      # CLI + live Jev + catalog + completion, offline fallback
│   ├── attest.rs    # TEE attestation report + nonce verify
│   ├── jev.rs       # Choice/Score/Noul types + confidence
│   ├── jev_wire.rs  # SystemOne JSON (request/response, serde)
│   ├── policy.rs    # budget x quality x sensitivity engine
│   ├── near.rs      # NEAR Cloud TEE vs proxied + ApiKey
│   ├── near_wire.rs # chat + ModelEntry catalog JSON, estimated_cents()
│   ├── ironclaw.rs  # channels, deploy target, caps, vault refs
│   ├── route.rs     # orchestrator + answers_from_wire() + estimate_plan()
│   ├── budget.rs    # client-controlled monthly caps
│   ├── http.rs      # bearer JSON POST/GET, key never logged
│   ├── live.rs      # evaluate()/complete()/list_models()/cheapest() via env keys
│   └── cli.rs       # brief + --monthly-cents + --live parsing
├── docs/
│   ├── NEWBIE.md    # 5-min start: demo, monthly, preset, API users
│   ├── KEYS.md      # key sources, rotation, no-hosting answer
│   └── SERVER.md    # per-tenant router-proxy design
├── .github/workflows/ci.yml
├── .gitignore
├── rustfmt.toml
├── README.md
├── CHANGELOG.md
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
├── SECURITY.md
├── LICENSE-MIT
└── LICENSE-APACHE
```

## Roadmap

1. Foundation (done): Cargo + lib + bin + docs
2. Licenses + contributing + conduct + security + CI + fmt (done)
3. `jev` router types (done): state, Choice/Score/Noul, confidence gating
4. Policy engine (done): budget x quality x sensitivity, `force_tee`
5. `near` types (done): TEE vs proxied, `ModelInfo`, redacting `ApiKey` — HTTP next
6. `ironclaw` types (done): channels, deploy target, caps, `VaultRef` — SSH/API next
7. Orchestrator `route()` (done): `Answers` + `route()` + TEE filter + caps check
8. Monthly budget (done): client-controlled `MonthlyBudget` + `route_with_monthly()`
9. Newbie guide (done): see `docs/NEWBIE.md` — demo, client-set cap, preset, API users
10. Wire JSON (done): `jev_wire` SystemOne + `near_wire` chat shapes via `serde`
11. HTTP helper (done): `http` bearer POST/GET via `ureq`, offline tests only
12. Live callers (done): `live` `evaluate()` + `complete()` via env keys, offline tests
13. CLI (done): `cli` brief + `--monthly-cents` + `--live`, wired into `main`
14. Live Jev loop (done): `main` evaluates Jev → `answers_from_wire()` → routes → completes, offline fallback
15. Live catalog (done): `list_models()` + `pick_best()` on `estimate_plan()` — cheapest price wins, ties prefer stronger models
16. Attestation (done): `attest` report types + nonce-bound `verify()` — full DCAP stays upstream
17. Server design (done): see `docs/SERVER.md` — per-tenant router-proxy, not built yet

## Docs

New here? Start with [`docs/NEWBIE.md`](docs/NEWBIE.md) — 5 minutes to your first routed answer.
Keys: [`docs/KEYS.md`](docs/KEYS.md) — where each key comes from, rotation, and why nothing needs hosting.
Server: [`docs/SERVER.md`](docs/SERVER.md) — `jear` as a per-tenant router-proxy (design, not built yet).

## Contributing

Slow, small, verified PRs. One file at a time, tests required.
See `CONTRIBUTING.md`.

## License

Dual-licensed under MIT or Apache-2.0. See `LICENSE-MIT` and `LICENSE-APACHE`.
