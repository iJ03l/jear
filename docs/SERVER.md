# Server mode — `jear` as a router-proxy for inference users

Same core as the CLI, behind one HTTP endpoint. Inference users change only
the base URL; routing, budgets, and privacy work per request exactly as
`main.rs` does them today. Nothing here is built yet — this is the design
the glue module must follow.

## Endpoint

`POST /v1/chat/completions` — the same OpenAI-compatible shape NEAR serves,
so existing clients migrate by pointing at `jear` instead:

- In: `{model: "auto", messages: [...], monthly_cents?: u64}` (`"auto"` means
  "you pick"; a concrete id pins the model and skips catalog picking).
- Out: standard chat completion **plus** `jear` fields: `route`, `reason`,
  `force_tee`, `usage`, and attestation material when the model is TEE-hosted.

## Per-request flow (same modules, tenant-scoped)

1. **Auth:** client key from the `Authorization` header — never env-shared,
   never logged (see `http::HttpError`, `near::ApiKey` redaction rules).
2. **Budget:** load the caller's `MonthlyBudget` (client-controlled limit);
   `route_with_monthly()` escalates to human review when the estimate exceeds
   the remainder — same as CLI.
3. **Route:** Jev `evaluate()` on the brief (1 parallel call: complexity,
   sensitivity, needs_private + confidences) → `answers_from_wire()`; offline
   `Answers` fallback when Jev is unreachable, exactly like `main.rs`.
4. **Pick:** `list_models()` → `pick_best()` on `estimate_plan(complexity)` —
   cheapest priced wins, ties prefer reasoning/tools/large context. Honor
   `force_tee` before pricing (TEE-only shortlist when set).
5. **Complete:** `complete()` on the winner; return answer + `usage` so the
   caller records spend against their monthly cap.
6. **Attest:** pass through TEE attestation/signature material untouched for
   the caller to verify (verify-before-display is still roadmap, not built).

## Tenant isolation (the one thing to get right)

- Keys, budgets, and briefs live and die inside one request scope. No globals,
  no shared `MonthlyBudget`, no cross-tenant catalog cache entries carrying
  auth material.
- `VaultRef` names only cross module boundaries — values are injected at the
  host boundary for allowlisted endpoints, per `ironclaw` rules.
- Rate-limit and timeout per tenant; upstream failures degrade to
  `HumanReview` with a reason, never a silent expensive call.

## Deploy notes

- Single binary behind any reverse proxy (TLS there). Concurrency and
  streaming passthrough are transport concerns — routing stays sync and pure.
- Start single-instance; the router holds no state except the caller's
  budget store, which must be shared/atomic if you scale horizontally.

## Non-goals

- No model hosting, no key minting, no billing ledger — NEAR/TypeSafe/Hub
  own those. `jear` routes, enforces caps, and passes proof through.
