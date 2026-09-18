# Newbie start — 5 minutes to your first routed answer

`jear` picks the model/agent for you by budget, quality, and sensitivity. No prompt engineering, no model picker.

## 1. Run the offline demo (no keys, 60 seconds)

```bash
cargo test   # 23 passed
cargo run
```

You should see:

```text
jear v0.1.0 — foundation ready
route: DirectLlm (tee: false)
reason: simple lookup — direct inference
eligible models: 2
caps ok for 5c: true
target: Hub("demo-instance")
```

That is the whole router working offline: Jev-style answers → policy → model filter → caps check.

## 2. Set a monthly cap ("no more than $100")

In code this is `MonthlyBudget::new(10_000)` ($100 = 10_000 cents):

- `remaining()` = limit − spent (never negative)
- `can_afford(estimate)` gates every route
- `route_with_monthly()` escalates to human review when over budget

Jev helps by answering `economical_model` (cheap-tee / balanced / frontier) + `cost_sensitivity` in the same 1 parallel call — cheapest passing route wins.

## 3. Go live (2 env vars, when ready)

```bash
export NEAR_API_KEY="sk-..."
export TYPESAFE_API_KEY="..."
```

- NEAR: `POST https://cloud-api.near.ai/v1/chat/completions` (OpenAI-compatible). TEE-hosted models keep prompts private with attestation; proxied models bill the same but without TEE guarantees.
- Jev: `POST /v1/systemone` with `state` + parallel `Choice`/`Score`/`Noul` questions. Output tokens are free (~$0.042/1M in).

## 4. IronClaw with the NEAR cloud preset (recommended)

1. Sign in at `agent.near.ai`, create an IronClaw instance (TEE enclave, no setup).
2. Add secrets to the encrypted vault — the model never sees raw values.
3. `jear` passes your `CostCaps` through to `max_daily_cost_cents` and shows the attestation ID back.
4. Prefer local? Run `ironclaw onboard` + `ironclaw serve`, then point `jear` at `DeployTarget::Local`.

## 5. Inference API users

Already calling NEAR? Keep your `POST /v1/chat/completions` shape. `jear` acts as a router proxy: same request in, Jev picks TEE vs proxied + cheap vs frontier, same billing out, plus `force_tee` filtering and a human-readable `reason`.

## FAQ

- **Do I need to choose a model?** No. Describe the task + monthly cap; Jev + policy choose.
- **What if confidence is low or I'm over budget?** You get `HumanReview` with a reason — never a silent expensive call.
- **Are my secrets safe?** `ApiKey` redacts on debug, `VaultRef` holds names only, `.env` is git-ignored. See `SECURITY.md`.
