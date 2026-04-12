# ADR-002: Security Audit and Hardening

Status: accepted
Date: 2025-07-22

## Context

A security-first audit of convergio-observatory identified several vulnerabilities
and code quality issues that needed remediation before production use.

## Findings

| # | Category | Severity | Location | Description |
|---|----------|----------|----------|-------------|
| 1 | SSRF | High | `export::register_webhook` | Webhook URL accepted without scheme or host validation; could target internal services |
| 2 | Info Leak | Medium | `routes::err_json` | Raw database/pool error messages returned to API clients |
| 3 | Prometheus Injection | Medium | `export::prometheus_exposition` | Label values not escaped; newlines/quotes could inject fake metrics |
| 4 | FTS5 Injection | Medium | `search::search` | User query passed raw to FTS5 MATCH; operators like NOT/OR alter semantics |
| 5 | MCP Schema Mismatch | Low | `mcp_defs` | Search tool declared `query` param but route expects `q`; webhook used `events` but API expects `event_filter`; timeline missing `source`/`node_id`/`until` |
| 6 | Missing Input Validation | Low | All routes | No length limits on query parameters |

## Decisions

### 1. Webhook URL validation (SSRF prevention)
- Require `https://` scheme (allow `http://localhost` for dev)
- Block private/reserved IP ranges (10.x, 192.168.x, 172.16-31.x, 169.254.x)
- Block cloud metadata endpoints (metadata.google, metadata.aws)
- Enforce 2048-char URL length limit

### 2. Error message sanitization
- `err_json()` now logs internal details via `tracing::warn!` but returns
  only a generic public message to the caller, keyed by error code.

### 3. Prometheus label escaping
- `sanitize_label_value()`: escapes `\`, `"`, and `\n` per the Prometheus
  exposition format specification.
- `sanitize_metric_name()`: strips non-alphanumeric characters (except `_` and `:`).

### 4. FTS5 query sanitization
- `sanitize_fts_query()`: wraps each whitespace-delimited token in double
  quotes, escaping embedded quotes. This neutralizes FTS5 operators
  (`AND`, `OR`, `NOT`, `NEAR`, `*`, `-`, `^`, `(`, `)`, `:`) so user
  input is always treated as literal text.

### 5. MCP schema alignment
- Fixed `cvg_observatory_search` to use `q` (matching `SearchQuery`).
- Fixed `cvg_create_webhook` to use `event_filter` (matching `WebhookRequest`).
- Added `source`, `node_id`, `until` to `cvg_observatory_timeline`.

### 6. Input length validation
- All query-string parameters capped at 512 characters via `validate_len()`.
- Empty search queries rejected.

## Test coverage

Added 7 new tests (19 → 26 total):
- `register_webhook_rejects_http`
- `register_webhook_rejects_private_ip`
- `register_webhook_allows_localhost_http`
- `prometheus_label_injection_escaped`
- `sanitize_fts_strips_operators`
- `sanitize_fts_empty`
- `sanitize_fts_handles_quotes`

## Consequences

- Webhook creation is now a stricter API (breaking change for non-HTTPS URLs).
- API error responses no longer expose internal details; debugging requires logs.
- FTS5 queries lose advanced operator support; users cannot use `AND`/`OR`/`NOT`.
  This is acceptable for a security-first posture; advanced search can be added
  behind an explicit opt-in flag later.
