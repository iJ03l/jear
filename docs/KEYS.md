# Keys — where each one comes from, how to use it, how to rotate it

`jear` works offline with zero keys. Live calls need keys, all passed as
environment variables so they never touch source control. Nothing here is
hosted by `jear` itself — see "Do I host anything?" below.

## `NEAR_API_KEY` — NEAR AI Cloud inference + catalog

- **Where:** sign in at the Developer Dashboard (`cloud.near.ai`), create an
  organization/workspace if asked, then create an API key. It starts with `sk-`.
- **Alternative:** stake NEAR to fund private inference and earn credits with
  no credit card — same dashboard, same `sk-` key shape.
- **Use:**
  ```bash
  export NEAR_API_KEY="sk-..."
  cargo run -- "brief in plain English" --monthly-cents 5000 --live
  ```
- **Optional override:** `NEAR_BASE_URL` (defaults to the documented
  `https://cloud-api.near.ai/v1`).

## `TYPESAFE_API_KEY` — Jev routing decisions

- **Where:** Jev (TypeSafe AI) is in early access — join the waitlist at
  `typesafe.ai`. Your key arrives with the invite.
- **Use:**
  ```bash
  export TYPESAFE_API_KEY="..."
  export TYPESAFE_BASE_URL="https://your-typesafe-endpoint"
  ```
  Both must be set: `jear` has no default Jev host hardcoded, so without the
  base URL it stays on offline answers by design.
- **Cost note:** Jev bills input tokens only (~$0.042/1M in our research);
  outputs are free, so routing every request stays cheap.

## IronClaw — agents via NEAR preset or local

- **Cloud preset (recommended):** sign in at `agent.near.ai`, add your SSH key
  (`ssh-keygen -t rsa -b 4096`, then paste `~/.ssh/id_rsa.pub`), create an
  IronClaw instance. It boots inside a TEE enclave. Add API tokens to the
  encrypted vault — the model never sees raw values. `jear` points at it with
  `DeployTarget::Hub("<instance-id>")` and passes your `CostCaps` through to
  `max_daily_cost_cents`.
- **Local:** install the single binary, run `ironclaw onboard`, then
  `ironclaw serve`. Point `jear` at `DeployTarget::Local`.

## Rotation (leaked or old keys)

1. Revoke/rotate at the provider first (dashboard, or new SSH key for Hub).
2. Update your shell env / secret manager — never commit `.env` (git-ignored).
3. If a key touched git history, purge it per `SECURITY.md` and rotate again.

## Do I host anything? No.

`jear` is a local Rust binary/library: `cargo run` on your machine, no
servers, no deploy. The hosted parts all belong to others — NEAR AI Cloud
(inference), TypeSafe API (Jev), IronClaw Hub (agents). `jear` only makes
HTTPS calls to them. Hosting `jear` itself is purely optional (e.g. as a
shared router-proxy for inference users speaking the same OpenAI shape).
