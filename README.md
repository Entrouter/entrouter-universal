# entrouter-universal

> **What goes in, comes out identical.**

Every layer in your stack has its own opinion about escaping.  
HTTP escapes it. JSON escapes it again. Redis adds its own twist. Postgres finishes the job.  
By the time your data arrives, it's been mangled by five different layers that all thought they were being helpful.

`entrouter-universal` solves this permanently.

**Base64 at entry. Opaque string through every layer. Decode at destination. Verify.**

No special characters. Nothing to escape. Nothing to double-escape. Every layer just moves a string it physically cannot touch.

---

## The Problem

```
Original:   hello "world" it's \fine\
After HTTP: hello \"world\" it\'s \\fine\\
After JSON: hello \\"world\\" it\\'s \\\\fine\\\\
After Redis: ...you get the idea
```

Senior devs have been chasing this bug for decades. The fix is one function call.

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
entrouter-universal = "0.1"
```

---

## Three Tools

### 1. Encode / Decode — The Core

The simplest form. Encode at entry, decode at exit. Nothing in between can touch it.

```rust
use entrouter_universal::{encode_str, decode_str};

// Entry point — encode once
let encoded = encode_str(r#"{"token":"abc\"def","user":"john's data"}"#);

// Pass `encoded` through HTTP, JSON, Redis, Postgres — whatever you want
// It's just a plain alphanumeric string. Nothing to escape.

// Exit point — decode once
let original = decode_str(&encoded).unwrap();
// Identical to what went in. Every time.
```

---

### 2. Envelope — Entry + Fingerprint + Exit in One

Wraps your data with a SHA-256 fingerprint. If anything mutated it in transit, the exit point tells you immediately.

```rust
use entrouter_universal::Envelope;

// ── Entry point ───────────────────────────────────────────
let env = Envelope::wrap(r#"winner_token: abc"123"\n special chars"#);

// Serialize to JSON — safe to store in Redis, Postgres, send over HTTP
let json = env.to_json().unwrap();
// {"d":"d2lubmVyX3Rva2VuOi...","f":"a3f1b2...","v":1}

// Pass json through every layer in your stack...

// ── Exit point ────────────────────────────────────────────
let received = Envelope::from_json(&json).unwrap();
let original = received.unwrap_verified().unwrap();
// If this succeeds → data is identical to what went in
// If this errors   → something touched it, and you know it
```

---

### 3. Guardian — Named Layer Checkpoints

Watches a value through your entire pipeline and tells you **exactly which layer** broke it.

```rust
use entrouter_universal::Guardian;

let mut g = Guardian::new(r#"my "sensitive" token\value"#);

// Pass g.encoded() through your pipeline
// At each layer, record a checkpoint with whatever the value looks like there

g.checkpoint("after_http",     &value_after_http);
g.checkpoint("after_json",     &value_after_json);
g.checkpoint("after_redis",    &value_after_redis);
g.checkpoint("after_postgres", &value_after_postgres);

// Find the exact layer that broke it
if let Some(violation) = g.first_violation() {
    println!("Broken at: {}", violation.layer);
}

// Or print the full pipeline report
println!("{}", g.report());
// ━━━━ Entrouter Universal Pipeline Report ━━━━
// Original fingerprint: a3f1b2c4...
// Overall intact: false
//
//   Layer 1: after_http     — ✅
//   Layer 2: after_json     — ✅
//   Layer 3: after_redis    — ❌ VIOLATED
//     Expected: a3f1b2c4...
//     Got:      9f8e7d6c...
//   Layer 4: after_postgres — ❌ VIOLATED
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// Or assert in tests — panics with exactly which layer failed
g.assert_intact();
```

---

## Why Base64

Base64 output contains only `A-Z a-z 0-9 + / =`.

None of these characters are special in:

| Layer | Safe? |
|---|---|
| HTTP headers | ✅ |
| JSON strings | ✅ |
| Rust strings | ✅ |
| Redis keys/values | ✅ |
| Postgres text columns | ✅ |
| URLs (with `+` → `-` variant) | ✅ |

Every layer just sees a boring alphanumeric string. There is nothing to escape. The problem is solved at the encoding level, not the escaping level.

---

## The Fingerprint

Each `Envelope` carries a SHA-256 fingerprint of the original raw input alongside the encoded data.

```
Original input → SHA-256 → fingerprint (travels with data)
Original input → Base64  → encoded    (travels with data)

At exit:
  decode(encoded) → SHA-256 → compare with fingerprint
  Match   → data is intact
  No match → something mutated it, and you know it
```

The fingerprint cannot be forged without knowing the original input. Any mutation — even a single character — produces a completely different SHA-256 hash.

---

## Real World Example — Entrouter Token Pipeline

```rust
use entrouter_universal::Envelope;

// Server generates a race winner token
let winner_token = format!(
    "{}-{}-{}-{}",
    utc_ms, drift_us, random_hex, user_id
);

// Wrap it at the entry point
let env = Envelope::wrap(&winner_token);

// Store in Redis — just a JSON string, nothing special
redis.set(&race_key, env.to_json()?).await?;

// Read from Redis
let stored = redis.get(&race_key).await?;
let env = Envelope::from_json(&stored)?;

// Write to Postgres — still just a JSON string
db.execute(
    "INSERT INTO race_results (token_envelope) VALUES ($1)",
    &[&env.to_json()?]
).await?;

// Read from Postgres and verify at the final destination
let row = db.query_one("SELECT token_envelope FROM race_results WHERE id = $1", &[&id]).await?;
let env = Envelope::from_json(row.get("token_envelope"))?;
let token = env.unwrap_verified()?;
// token == winner_token, guaranteed, or it errors
```
