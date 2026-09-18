# Contributing to jear

Thanks for helping build a Jev-routed client for everyone. We move slowly and build it great: one file at a time, verified.

## Ground rules

1. Small PRs — one concern per PR, one file at a time when possible.
2. Tests required — `cargo test` must pass. Add a test for new logic.
3. No secrets — never commit `.env`, API keys (`sk-`, `TYPESAFE_API_KEY`), or private prompts.
4. Verified 3x — read back your file, check `sha256sum`, run the build.

## Setup

```bash
cargo test
cargo run
cargo fmt --check
cargo clippy -- -D warnings
```

Requires Rust 1.70+. Tested on 1.96.0.

## Workflow

1. Open an issue describing budget / quality / sensitivity impact.
2. Implement in `src/` with docs (`///`) on public items.
3. Run:
   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   cargo test
   ```
4. Open PR with: what changed, how verified (3 checks), cost/latency impact if router-related.

## Code style

- `cargo fmt` clean, no warnings from `clippy`.
- Public APIs need doc comments with an example.
- Prefer `Result` over `panic` outside `main`. No `unwrap` in library code.
- Keep dependencies minimal — justify every new crate.

## Commit messages

- Format: `area: short imperative description`
- Examples: `router: add Jev Choice types`, `near: list models adapter`, `docs: clarify TEE guarantees`

## License

By contributing you agree your work is dual-licensed as `MIT OR Apache-2.0` (see `LICENSE-MIT`, `LICENSE-APACHE`).
