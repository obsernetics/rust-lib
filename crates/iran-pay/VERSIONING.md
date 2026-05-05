# Versioning policy

`iran-pay` follows [SemVer 2.0](https://semver.org) **strictly for the
trait-level API** but treats the wire-level provider drivers with extra
caution because Iranian payment gateways change their HTTP APIs without
public notice.

## What's covered by SemVer

| Surface                                         | SemVer guarantee                              |
|-------------------------------------------------|-----------------------------------------------|
| `Gateway` trait                                 | **Stable** — breaking changes only in MAJOR.  |
| `StartRequest`, `VerifyResponse`, `Amount`, `Error` | **Stable** — additive only in MINOR.        |
| `MockGateway`, `security` helpers               | **Stable** — additive only in MINOR.         |
| `validators` re-exports                         | Tied to the underlying `parsitext` version.   |
| Per-provider driver structs (`ZarinPal`, …)     | **Constructor-stable** — `new`, `sandbox`, `with_*` builders never break. |
| Per-provider driver wire format                 | **Best-effort.** See below.                   |

## What's not strictly SemVer-covered

The actual JSON / form-encoded body shape that each driver puts on the
wire is dictated by the upstream provider, which **we do not control**.
When a provider silently changes a field name, removes an endpoint, or
revs their API version:

- We treat it as a **bug**, not a breaking change.
- The fix ships in the next **PATCH** release (e.g. `0.1.x` → `0.1.x+1`).
- We **may not** bump the major version even if the change is
  technically observable (e.g. a new field in `StartResponse.raw`).

If you depend on the *exact bytes* in `StartResponse.raw` or
`VerifyResponse.raw`, treat them as **opaque diagnostic data**, not
public API.

## Provider API versions pinned by this crate

Every driver targets a specific upstream API version.  We update these
in PATCH releases when providers ship breaking changes; the table below
is the source of truth.

| Driver       | Upstream version | Endpoint base                       | Verified                       |
|--------------|------------------|-------------------------------------|--------------------------------|
| `ZarinPal`   | v4               | `https://payment.zarinpal.com`      | docs.zarinpal.com (2026-05)    |
| `IDPay`      | v1.1             | `https://api.idpay.ir`              | idpay.ir/web-service/v1.1 (2026-05) |
| `NextPay`    | (un-versioned)   | `https://nextpay.org`               | nextpay.org/nx/docs (2026-05)  |
| `PayIr`      | (un-versioned)   | `https://pay.ir`                    | docs.pay.ir/gateway (2026-05)  |
| `Zibal`      | v1               | `https://gateway.zibal.ir`          | github.com/zibalco (2026-05)   |
| `Vandar`     | v3               | `https://ipg.vandar.io`             | vandarpay.github.io/docs (2026-05) |

## Cargo MSRV

Minimum supported Rust version: **1.85**.  Bumping the MSRV is treated
as a MINOR change, never a PATCH change.

## How to upgrade safely

1. **Pin to a tilde range** in your `Cargo.toml`:
   ```toml
   iran-pay = "~0.1"
   ```
   This accepts patch updates (`0.1.x`) automatically — exactly what we
   ship provider-API fixes in.
2. **Pin TLS / provider features** explicitly so you control your
   binary surface:
   ```toml
   iran-pay = { version = "~0.1", default-features = false,
                features = ["zarinpal", "idpay", "rustls-tls", "validators"] }
   ```
3. **Run integration tests against your providers' sandboxes** in CI.
   Your tests catch upstream breakage before your customers do.
4. **Watch the changelog** — every PATCH release names the gateway it
   fixed.

## Reporting upstream API changes

If you observe a gateway rejecting or accepting requests that
contradict this crate, please open an issue with:

- Driver name (`zarinpal`, etc.).
- The exact request body the driver sent (set up
  `tracing_subscriber::fmt()` to capture).
- The exact response body.
- Where you found the contradicting documentation.

Most fixes ship within 24 hours of a verified report.
