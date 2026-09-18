# Security Policy

jear routes sensitive prompts to private inference and secure agents. Security is a first-class feature, not an add-on.

## Supported versions

| Version | Supported |
| ------- | --------- |
| 0.1.x   | Yes — foundation, security reports accepted |

## Reporting a vulnerability

Do not open a public issue for vulnerabilities.

1. Use GitHub private security advisory for this repo, or email maintainers.
2. Include: affected version, steps to reproduce, impact (especially TEE / vault / routing bypass), and suggested fix if any.
3. Expect acknowledgment within 72 hours. We aim to fix critical issues within 14 days.

## Secrets handling

- Never commit `.env`, `sk-` keys, `TYPESAFE_API_KEY`, SSH keys, or user prompts with PII.
- `jear` will: prefer TEE-hosted NEAR models for sensitive state, verify attestation before display, and inject IronClaw secrets only at the host boundary.
- If you leak a key: rotate immediately, then notify maintainers to purge history.

## Scope

In scope: router confidence-gating bypass, budget over-spend, private-to-public model fallback, attestation verification failure, IronClaw vault / sandbox escape via `jear`.

Out of scope (report to upstream): NEAR AI Cloud TEE itself, IronClaw core, Jev model behavior — but tell us so we can pin / work around.

Thank you for keeping `jear` safe for everyone.
