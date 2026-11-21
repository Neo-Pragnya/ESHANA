# Security policy

## Reporting

Please report vulnerabilities privately through GitHub's *Report a vulnerability* (Security tab of
`Neo-Pragnya/ESHANA`) rather than a public issue. Include the version, a minimal request and the impact. You will get an
acknowledgement within a few days.

## Scope and posture

ESHANA is a local computation library: no network, no telemetry, no files written, no secrets. The realistic risks are:

- **Untrusted input** (request JSON, names, zone strings, kernel files). Inputs are validated; unknown fields are refused;
  the code forbids `unsafe`; the time and text parsers are property-tested for panics. The SPK reader bounds-checks every
  read and rejects malformed or foreign files with `ESH-PROVIDER`. If you can make it panic, that is a bug to report.
- **Supply chain.** `cargo deny` enforces a licence allow-list and bans unknown registries; `cargo audit` and a distribution
  scan of the built wheel run in CI.
- **Personal data.** Birth data is personal data; see `docs/concepts/privacy-and-ethics.md`. The engine retains nothing.

Not a security matter: disagreement with a tradition's interpretation, or a position that differs from another program's
within the documented accuracy.
