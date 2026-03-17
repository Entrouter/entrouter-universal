# entrouter-universal

> **What goes in, comes out identical.**

Every layer in your stack has its own opinion about escaping.  
HTTP. JSON. Rust. Redis. Postgres. Each one mangling your data a little more.  
By the time it arrives, it's unrecognisable.

`entrouter-universal` solves this permanently.

**Base64 at entry. Opaque to every layer. SHA-256 fingerprint travels with it. Verify at exit.**

---

## Installation

```toml
[dependencies]
entrouter-universal = "0.2"

# Optional: compression support
entrouter-universal = { version = "0.2", features = ["compression"] }
```

---

## Five Tools

### 1. `Envelope` — Four Wrap Modes

#### Standard
```rust
let env = Envelope::wrap(r#"{"token":"abc\"def","user":"john's"}"#);
// pass through HTTP → JSON → Redis → Postgres
let original = env.unwrap_verified().unwrap(); // identical or Err
```

#### URL-Safe
```rust
// Use in URLs, query params, HTTP headers
// Uses - and _ instead of + and /. No padding. Zero breakage.
let env = Envelope::wrap_url_safe(data);
assert!(env.d.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
let original = env.unwrap_verified().unwrap();
```

#### Compressed
```rust
// Large payloads — gzip first, then Base64
// Smaller on the wire. Transparent to consumer.
let env = Envelope::wrap_compressed(large_payload)?;
let original = env.unwrap_verified().unwrap(); // auto-decompresses
```

#### TTL — Self-Expiring Data
```rust
// Race tokens, session data, anything time-sensitive
let env = Envelope::wrap_with_ttl(data, 300); // expires in 5 minutes

println!("{} secs remaining", env.ttl_remaining().unwrap());

// After 5 minutes:
env.unwrap_verified() // Err(Expired { expired_at: ..., now: ... })
```

---

### 2. `Chain` — Cryptographic Audit Trail

Each link references the previous link's fingerprint. Tamper with any link — everything after it breaks. You know exactly where the chain was cut.

```rust
let mut chain = Chain::new("race:listing_abc — OPENED");
chain.append("user_john joined — token: 000001739850000001");
chain.append("user_jane joined — token: 000001739850000002");
chain.append("WINNER: user_john");
chain.append("race:listing_abc — CLOSED");

// Verify the entire chain
let result = chain.verify();
assert!(result.valid);
println!("{}", chain.report());
// ━━━━ Entrouter Universal Chain Report ━━━━
// Links: 5 | Valid: true
//   Link 1: ✅ | ts: 1739850000 | fp: a3f1b2c4d5e6f7...
//   Link 2: ✅ | ts: 1739850001 | fp: 9f8e7d6c5b4a3...
//   ...

// Tamper with link 3 — links 4 and 5 break too
chain.links[2].d = encode_str("TAMPERED");
let result = chain.verify();
assert_eq!(result.broken_at, Some(3));

// Store anywhere — Redis, Postgres, S3
let json = chain.to_json().unwrap();
let restored = Chain::from_json(&json).unwrap();
assert!(restored.verify().valid);
```

---

### 3. `UniversalStruct` — Per-Field Integrity

Wraps every field individually. If Redis mangles just one field, you know exactly which one. Not "something broke" — "`amount` was tampered with between Redis and Postgres."

```rust
let wrapped = UniversalStruct::wrap_fields(&[
    ("token",      "000001739850123456-000004521890000-a3f1b2-user_john"),
    ("user_id",    "john's account \"special\""),
    ("amount",     "299.99"),
    ("listing_id", "listing:abc\\123"),
]);

// Verify all fields
let result = wrapped.verify_all();
assert!(result.all_intact);

// Get specific field
let token = wrapped.get("token").unwrap();

// Get all as HashMap
let map = wrapped.to_map().unwrap();

// Simulate financial data tampering
wrapped.fields[2].d = encode_str("999999.99"); // mutate amount

let result = wrapped.verify_all();
assert!(!result.all_intact);
assert!(result.violations.contains(&"amount".to_string()));
// token ✅  user_id ✅  amount ❌  listing_id ✅

println!("{}", wrapped.report());
// ━━━━ Entrouter Universal Field Report ━━━━
// All intact: false
//   token:      ✅ — 000001739850123456...
//   user_id:    ✅ — john's account "special"
//   amount:     ❌ VIOLATED — —
//   listing_id: ✅ — listing:abc\123
```

---

### 4. `Guardian` — Named Layer Checkpoints

Find the exact layer that broke your data.

```rust
let mut g = Guardian::new(data);
let encoded = g.encoded().to_string(); // pass this through your pipeline

g.checkpoint("http_ingress",    &value_at_http);
g.checkpoint("json_parse",      &value_at_json);
g.checkpoint("redis_write",     &value_at_redis);
g.checkpoint("postgres_write",  &value_at_postgres);

// Exact culprit
if let Some(v) = g.first_violation() {
    println!("Broken at: {}", v.layer); // "redis_write"
}

println!("{}", g.report());
// ━━━━ Entrouter Universal Pipeline Report ━━━━
//   Layer 1: http_ingress   — ✅
//   Layer 2: json_parse     — ✅
//   Layer 3: redis_write    — ❌ VIOLATED
//   Layer 4: postgres_write — ❌ VIOLATED
```

---

### 5. Core Primitives

```rust
use entrouter_universal::{encode_str, decode_str, fingerprint_str, verify};

let encoded     = encode_str(data);      // Base64
let decoded     = decode_str(&encoded)?; // back to string
let fp          = fingerprint_str(data); // SHA-256 hex
let result      = verify(&encoded, &fp)?; // decode + verify in one call
```

---

## Cross-Machine — Works Everywhere

Both boxes just need the crate. Base64 and SHA-256 are universal standards — identical output on every OS, every architecture.

```
Windows PC                      Ubuntu VPS
cargo add entrouter-universal   cargo add entrouter-universal

Envelope::wrap(data)     →SSH→  Envelope::from_json(wire)
                                .unwrap_verified() // ✅ identical
```

Data wrapped on your dev machine arrives verified on your VPS. Same crate. Same standard. Guaranteed.

---

## API Reference

### `Envelope`
| Method | Description |
|---|---|
| `Envelope::wrap(input)` | Standard Base64 wrap |
| `Envelope::wrap_url_safe(input)` | URL-safe Base64 (- and _) |
| `Envelope::wrap_compressed(input)` | Gzip then Base64 |
| `Envelope::wrap_with_ttl(input, secs)` | Standard + expiry |
| `env.unwrap_verified()` | Decode + verify (all modes) |
| `env.unwrap_raw()` | Decode without verification |
| `env.is_expired()` | Check TTL |
| `env.ttl_remaining()` | Seconds until expiry |
| `env.to_json()` / `Envelope::from_json()` | Serialise |

### `Chain`
| Method | Description |
|---|---|
| `Chain::new(data)` | Start a new chain |
| `chain.append(data)` | Add a link referencing previous |
| `chain.verify()` | Verify entire chain |
| `chain.report()` | Human-readable report |
| `chain.to_json()` / `Chain::from_json()` | Serialise |

### `UniversalStruct`
| Method | Description |
|---|---|
| `UniversalStruct::wrap_fields(&[("name", "value")])` | Wrap field pairs |
| `struct.verify_all()` | Per-field verification |
| `struct.get("field")` | Get verified field by name |
| `struct.to_map()` | All fields as HashMap |
| `struct.assert_intact()` | Panic with field names if violated |
| `struct.report()` | Human-readable field report |

### `Guardian`
| Method | Description |
|---|---|
| `Guardian::new(input)` | Create at entry point |
| `guardian.encoded()` | Get encoded string for pipeline |
| `guardian.checkpoint(layer, current)` | Record named checkpoint |
| `guardian.first_violation()` | First broken layer |
| `guardian.is_intact()` | All checkpoints passed |
| `guardian.assert_intact()` | Panic with layer name |
| `guardian.report()` | Full pipeline report |

---

## License

**AGPL-3.0-only** — Free for open-source.  
Commercial license for closed-source / proprietary use.

Contact **entropy-router@proton.me**

---

*Part of the Entrouter suite — [entrouter.com](https://entrouter.com)*
