# jear

Jev-routed client for NEAR AI Cloud inference and IronClaw agents.

Users never pick a model or agent. `jear` routes by **budget**, **quality**, and **sensitivity** using Jev (TypeSafe AI System One) for fast, structured decisions.

- Status: `v0.1.0` core routing + wire JSON + HTTP helper — demo offline, live via env keys.
- Stack: Rust 1.70+ (tested on 1.96.0), `serde`/`serde_json`/`ureq`.
- License: `MIT OR Apache-2.0` — free for everyone.

## Why

- Jev decides: `Choice` (which agent), `Score` (complexity/sensitivity), `Noul` (needs private TEE? risky? urgent?) with calibrated confidence.
- NEAR AI Cloud executes: OpenAI-compatible `https://cloud-api.near.ai/v1`, three privacy tiers (TEE-only → TEE/anonymized → any) filtering the catalog, function tools forwarded verbatim — `jear` routes the input, outputs and tool calls flow normally.
- IronClaw acts: secure Rust agent OS, personal instances + Agent Marketplace hires.

## Start (full steps)

0. **Prereqs:** Rust 1.70+ (`rustup --version`), git, no keys needed yet.
1. **Clone & enter:**
   ```bash
   git clone https://github.com/iJ03l/jear.git
   cd jear
   ```
2. **Check everything passes:**
   ```bash
   cargo test
   ```
   Expect `60 passed` across unit + integration tests (`tests/routing.rs` drives the full Jev→route→catalog loop offline, including tool calls and tier filtering).
3. **Run the offline router (no keys, no network):**
   ```bash
   cargo run -- "brief in plain English" --monthly-cents 5000
   ```
   Expect: `route: DirectLlm`, a `reason`, `eligible models: 2`, your `monthly` remainder.
4. **Set your own monthly cap** (client-controlled, any amount in cents):
   ```bash
   cargo run -- "summarize my inbox" --monthly-cents 12000
   ```
5. **Go live with NEAR** (needs key, else stays offline with guidance):
   ```bash
   export NEAR_API_KEY="sk-..."
   cargo run -- "brief in plain English" --monthly-cents 5000 --live
   ```
6. **Add Jev routing on top** (both required — without the base URL `jear`
   stays on offline answers by design, NEAR completion still works):
   ```bash
   export TYPESAFE_API_KEY="..."
   export TYPESAFE_BASE_URL="https://your-typesafe-endpoint"
   cargo run -- "brief in plain English" --monthly-cents 5000 --live
   ```

Key sources: [`docs/KEYS.md`](docs/KEYS.md).

## Project layout

```text
jear/
├── Cargo.toml
├── src/
│   ├── lib.rs       # library root + module wiring
│   ├── main.rs      # CLI + live Jev + catalog + verify-before-display, offline fallback
│   ├── attest.rs    # TEE attestation report + nonce verify (fail-closed in --live)
│   ├── jev.rs       # Choice/Score/Noul types + confidence
│   ├── jev_wire.rs  # SystemOne JSON (request/response, serde)
│   ├── policy.rs    # budget x quality x sensitivity engine
│   ├── near.rs      # TEE/anonymized/proxied tiers + ApiKey
│   ├── near_wire.rs # chat + tools passthrough + ModelEntry catalog, estimated_cents()
│   ├── ironclaw.rs  # channels, deploy target, caps, vault refs
│   ├── route.rs     # orchestrator + answers_from_wire() + estimate_plan() + tier filter
│   ├── budget.rs    # client-controlled monthly caps
│   ├── http.rs      # bearer JSON POST/GET, key never logged
│   ├── live.rs      # evaluate()/complete()/list_models()/attestation_report() via env keys
│   └── cli.rs       # brief + --monthly-cents + --live parsing
├── tests/
│   └── routing.rs   # end-to-end routing loop, offline, no assumptions
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

## Docs

New here? Start with [`docs/NEWBIE.md`](docs/NEWBIE.md) — 5 minutes to your first routed answer.
Keys: [`docs/KEYS.md`](docs/KEYS.md) — where each key comes from, rotation, and why nothing needs hosting.
Server: [`docs/SERVER.md`](docs/SERVER.md) — `jear` as a per-tenant router-proxy (design, not built yet).

## Contributing

Slow, small, verified PRs. One file at a time, tests required.
See `CONTRIBUTING.md`.

## License

Dual-licensed under MIT or Apache-2.0. See `LICENSE-MIT` and `LICENSE-APACHE`.
