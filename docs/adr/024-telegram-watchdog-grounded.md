---
version: "1.0"
last_updated: "2026-04-07"
author: "convergio-team"
tags: ["adr"]
---

# ADR-024: Telegram Watchdog — Grounded Responses

**Status:** Accepted
**Date:** 2026-04-05
**Deciders:** Roberto D'Angelo

## Context

The Telegram bot acts as a remote watchdog for Convergio. Users send natural
language queries ("Stato?", "Quanti agenti attivi?") and expect answers based
on real system data. The bot uses a local MLX model (Qwen 7B) for natural
language understanding.

### Problem: hallucinated answers

When asked "Stato?", MLX generated a plausible but entirely fabricated response
including a fictional company name, fake metrics, and invented project details.
The model had no access to real system data and filled the gap with hallucination.

Example of hallucinated output:
```
Stato attuale di Convergio Srl:
- Fatturato Q1: €2.3M (+12%)
- Dipendenti: 47
- Progetti attivi: 12
```

None of this data exists. The bot must never invent system state.

## Decision

### Architecture: keyword match BEFORE MLX

```text
User message
    │
    ▼
┌─────────────────┐
│ Keyword matcher  │──match──> Direct API call ──> formatted response
│ (exact patterns) │
└────────┬────────┘
         │ no match
         ▼
┌─────────────────┐
│ fetch_system_   │──> Collect real data from daemon API
│ context()       │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ MLX inference   │──> "Answer ONLY based on this data: {context}"
│ (grounded)      │
└─────────────────┘
```

### Layer 1 — Keyword matching

Common queries are handled without MLX at all:

| Pattern | Action | Response source |
|---------|--------|----------------|
| `stato`, `status` | `GET /api/health` + `GET /api/agents` | Formatted health + agent list |
| `piani`, `plans` | `GET /api/plans` | Plan list with status counts |
| `mesh`, `nodi` | `GET /api/mesh/peers` | Peer table with transport/latency |
| `aiuto`, `help` | Static | Command reference |

Keyword matching is case-insensitive, accent-normalized, and checks word
boundaries. This layer handles ~70% of real queries with zero inference cost.

### Layer 2 — Grounded MLX inference

For queries that don't match keywords, the bot fetches real system context
before calling MLX:

```rust
async fn fetch_system_context(client: &HttpClient) -> String {
    let health = client.get("/api/health").await;
    let plans  = client.get("/api/plans?limit=5").await;
    let agents = client.get("/api/agents?status=active").await;
    let peers  = client.get("/api/mesh/peers").await;
    format!(
        "System health: {health}\n\
         Recent plans: {plans}\n\
         Active agents: {agents}\n\
         Mesh peers: {peers}"
    )
}
```

The MLX prompt is strictly bounded:

```text
You are Convergio assistant. Answer the user question ONLY based on
the system data below. If the data does not contain the answer, say
"Non ho dati sufficienti per rispondere."

SYSTEM DATA:
{context}

USER QUESTION:
{question}
```

### Chat template enforcement

MLX models require `apply_chat_template()` to produce correct token sequences.
Without it, the model sees raw text and leaks special tokens (`<|im_start|>`,
`<|endoftext|>`) into the output.

```python
tokens = tokenizer.apply_chat_template(
    [{"role": "system", "content": system_prompt},
     {"role": "user",   "content": user_question}],
    add_generation_prompt=True
)
```

This is mandatory for all MLX inference calls, not just Telegram.

### Telegram HTML escaping

Telegram `parse_mode=HTML` requires escaping `<`, `>`, `&` in all dynamic
content. A `telegram_escape()` helper sanitizes every field before sending:

```rust
fn telegram_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
}
```

Unescaped `<` in agent names or log output caused Telegram to reject messages
with "Bad Request: can't parse entities".

## Alternatives Considered

| Alternative | Rejected because |
|-------------|-----------------|
| MLX for all queries | Hallucination risk on factual questions |
| RAG with embeddings | Over-engineered for structured API data |
| Disable MLX entirely | Loses natural language flexibility for open questions |
| JSON parse_mode | Less readable for status tables |

## Consequences

- Keyword queries return in <100ms (no inference). MLX queries take ~1-2s
- New keyword patterns can be added without retraining or redeploying MLX
- `fetch_system_context()` adds 4 API calls per MLX query (~50ms total)
- If daemon is unreachable, bot returns "Daemon non raggiungibile" (no MLX)
- Chat template is enforced project-wide — prevents token leak in any MLX use
- HTML escaping is mandatory for all Telegram responses
