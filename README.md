<div align="center">

# Entrouter-Universal

### Pipeline Integrity Guardian

[![Crates.io](https://img.shields.io/crates/v/entrouter-universal?style=flat-square&color=fc8d62)](https://crates.io/crates/entrouter-universal)
[![Downloads](https://img.shields.io/crates/d/entrouter-universal?style=flat-square&color=66c2a5)](https://crates.io/crates/entrouter-universal)
[![License](https://img.shields.io/crates/l/entrouter-universal?style=flat-square&color=8da0cb)](https://github.com/Entrouter/entrouter-universal/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.63%2B-orange?style=flat-square)](https://www.rust-lang.org)
[![docs.rs](https://img.shields.io/docsrs/entrouter-universal?style=flat-square&color=e78ac3)](https://docs.rs/entrouter-universal)

**What goes in, comes out identical. Or it tells you exactly what broke and where.**

```
cargo install entrouter-universal
```

</div>

---

## The 20-Minute Problem, Fixed In 1 Second

You're SSH'd into your VPS. You need to run a curl command with JSON. You know what happens next.

<table>
<tr>
<td width="50%">

**Before - The escaping nightmare**

```
$ ssh root@your-vps

# Attempt 1
curl -d '{"key":"val"}' ...
> bash: unexpected EOF

# Attempt 2
curl -d "{\"key\":\"val\"}" ...
> bash: unexpected token

# Attempt 3
curl -d '{\"key\":\"val\"}' ...
> invalid JSON

# Attempt 4
curl -d '{"key":"val"}' ...
> curl: (3) bad/illegal format

  ...20 minutes later...
```

</td>
<td width="50%">

**After - One command**

```
$ echo 'curl -s -X POST \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer er_xxx" \
  -d {"tier":"enterprise"} \
  http://127.0.0.1:3000/admin/keys/generate' \
  | entrouter ssh root@your-vps

{"apiKey":"er_ent_9f3a...","tier":"enterprise"}

# Done. First try. Every time.


  
```

</td>
</tr>
</table>

You type the command **exactly as you would locally**. Entrouter encodes it to safe base64, sends it over SSH, decodes on the server, executes it. Zero escaping. Zero thinking.

---

## How It Works

```
┌──────────────┐          ┌───────────────────────┐          ┌──────────────┐
│  YOUR SHELL  │          │     SSH TRANSPORT      │          │  REMOTE VPS  │
│              │  encode   │                       │  decode   │              │
│  Raw command ├──────────►│  Safe base64 payload  ├──────────►│  Exact same  │
│  with JSON,  │  SHA-256  │  Nothing to escape.   │  verify   │  command you │
│  quotes, etc │  fingerprint  Nothing to break.   │  execute  │  typed.      │
└──────────────┘          └───────────────────────┘          └──────────────┘
```

**Base64 at entry. SHA-256 fingerprint travels with it. Verify at exit.**

Every layer between your keyboard and the destination - HTTP, JSON, Rust, Redis, Postgres, shell - sees a boring alphanumeric string. Nothing to escape. Nothing to mangle. Nothing to break.

---

## Quick Start

```bash
# Install the CLI
cargo install entrouter-universal

# Or add the library to your project
cargo add entrouter-universal
```

---

## CLI Commands

Eleven commands. All pipe-friendly. All shell-safe.

| Command | What it does |
|---|---|
| `entrouter ssh <host>` | Pipe a command in, it runs on the remote machine. No escaping. |
| `entrouter multi-ssh <h1,h2,...>` | Same thing, but fan-out to multiple hosts in parallel. |
| `entrouter docker <container>` | Pipe a command in, it runs inside the Docker container. |
| `entrouter kube <pod> [-n ns]` | Pipe a command in, it runs inside the Kubernetes pod. |
| `entrouter cron [schedule]` | Encode a command into a cron-safe line. No `%` breakage. |
| `entrouter exec` | Decode base64 from stdin and execute it locally. |
| `entrouter encode` | Stdin -> base64 + SHA-256 fingerprint (JSON output) |
| `entrouter decode` | JSON with encoded field -> original data |
| `entrouter verify` | JSON with encoded + fingerprint -> INTACT or TAMPERED |
| `entrouter raw-encode` | Stdin -> raw base64 (no JSON wrapper) |
| `entrouter raw-decode` | Base64 -> original (no JSON wrapper) |
| `entrouter mcp` | Start the MCP server (for AI agents in VS Code, Cursor, etc.) |

---

## MCP Server - AI Agent Integration

Entrouter ships with a built-in [MCP (Model Context Protocol)](https://modelcontextprotocol.io/) server with **21 tools**. This lets AI agents like GitHub Copilot, Claude, or any MCP-compatible client use entrouter's tools directly — encoding, decoding, fingerprinting, integrity checks, HMAC-signed envelopes, cryptographic audit chains, and **running commands on remote servers, Docker containers, Kubernetes pods, or multiple hosts in parallel** without any shell escaping issues.

### Setup (2 minutes)

**1. Install entrouter**

```bash
cargo install entrouter-universal
```

**2. Add to VS Code**

Open your VS Code settings JSON and add:

<table>
<tr>
<td width="50%">

**Global** (all workspaces)

Edit `settings.json` → add:
```json
{
  "mcp": {
    "servers": {
      "entrouter": {
        "type": "stdio",
        "command": "entrouter",
        "args": ["mcp"]
      }
    }
  }
}
```

</td>
<td width="50%">

**Per-workspace**

Create `.vscode/mcp.json`:
```json
{
  "servers": {
    "entrouter": {
      "type": "stdio",
      "command": "entrouter",
      "args": ["mcp"]
    }
  }
}
```

</td>
</tr>
</table>

That's it. Restart VS Code and the tools are available to your AI agent.

### Available MCP Tools

**Core (encoding & integrity)**

| Tool | What it does |
|---|---|
| `entrouter_encode` | Encode text → base64 + SHA-256 fingerprint (JSON output) |
| `entrouter_decode` | Decode base64 back to original text |
| `entrouter_verify` | Check data integrity — INTACT or TAMPERED |
| `entrouter_raw_encode` | Encode text → plain base64 (no wrapper) |
| `entrouter_raw_decode` | Decode plain base64 → original text |
| `entrouter_fingerprint` | Compute SHA-256 fingerprint of any text |

**Envelopes**

| Tool | What it does |
|---|---|
| `entrouter_envelope_wrap` | Wrap data in an envelope (standard, url-safe, compressed, or TTL mode) |
| `entrouter_envelope_unwrap` | Unwrap and verify an envelope — checks integrity + expiry |
| `entrouter_signed_wrap` | Wrap data with HMAC-SHA256 authentication — proves integrity AND origin |
| `entrouter_signed_unwrap` | Unwrap a signed envelope — verifies HMAC signature + integrity |

**Chains (cryptographic audit trails)**

| Tool | What it does |
|---|---|
| `entrouter_chain_new` | Create a new chain with a genesis link |
| `entrouter_chain_append` | Append a link to an existing chain |
| `entrouter_chain_verify` | Verify the entire chain — find exactly where it broke |
| `entrouter_chain_diff` | Compare two chains — find where they diverge |
| `entrouter_chain_merge` | Merge two chains (one must be a prefix of the other) |

**Remote execution**

| Tool | What it does |
|---|---|
| `entrouter_ssh` | Run any command on a remote server via SSH — no escaping needed |
| `entrouter_multi_ssh` | Run a command on multiple hosts in parallel — collect all results |
| `entrouter_docker` | Run a command inside a Docker container — no `docker exec` escaping |
| `entrouter_kube` | Run a command inside a Kubernetes pod — no `kubectl exec` escaping |

### SSH Tool Requirements

The `entrouter_ssh` tool lets your AI agent run commands on remote servers. For it to work:

1. **SSH key access** — your machine must be able to `ssh user@host` without a password prompt
2. **Entrouter on the remote** — install on the remote server too: `cargo install entrouter-universal`

Once set up, your AI agent can do things like:
- Check server health: `curl -s http://localhost:3000/health`
- Restart services: `systemctl restart my-service`
- Read logs: `journalctl -u my-service --no-pager -n 50`
- Run database queries, deploy code, debug production — anything you'd type in a terminal

All commands are base64-encoded locally, sent over SSH, decoded on the remote, and executed. No escaping. 30-second timeout prevents hangs.

---

### Multi-SSH - Fan-Out To N Hosts

```bash
# Run a command on multiple servers at once. Results come back labeled.
echo 'uptime' | entrouter multi-ssh root@server1,root@server2,root@server3

# Health check all your VPS instances
echo 'curl -s http://localhost:3000/health' | entrouter multi-ssh root@vps1,root@vps2
```

Same encoding, same zero-escaping — just comma-separate the hosts. Results return per-host.

---

### SSH - The Killer Feature

```bash
# Run ANY command on a remote server. Type it exactly how you would locally.
echo 'curl -s -X POST -H "Content-Type: application/json" -d {"key":"value"} http://localhost:3000/api' | entrouter ssh root@your-vps
```

No quoting gymnastics. No backslash hell. No "it works locally but breaks over SSH." Just pipe your command and go.

### Docker - Same Pain, Same Fix

```bash
# Run ANY command inside a container. No docker exec escaping hell.
echo 'nginx -t && nginx -s reload' | entrouter docker my-nginx

# Complex JSON, special chars - doesn't matter
echo 'curl -X POST -d {"config":"new_value"} http://localhost:8080/api' | entrouter docker my-app
```

Uses `base64 -d` on the container side - zero dependencies. Works with any image.

### Kubernetes - The Worst Escaping Of All

```bash
# Run a command in a pod
echo 'cat /etc/config/app.yaml' | entrouter kube my-pod

# Specify namespace
echo 'pg_dump -U postgres mydb > /tmp/backup.sql' | entrouter kube db-pod -n production
```

No more `kubectl exec my-pod -- sh -c "..."` with triple-escaped quotes.

### Cron - Kill The % Problem

Cron interprets `%` as a newline. Date formats, URL-encoded strings, modulo ops - all break silently.

```bash
# Generate a cron-safe line
echo 'date +%Y-%m-%d | tee /var/log/daily.log' | entrouter cron '0 2 * * *'
# 0 2 * * * echo 'ZGF0ZSArJVk...' | base64 -d | sh

# Without schedule - just the encoded execution part
echo 'backup.sh --format=%s' | entrouter cron
# echo 'YmFja3VwLn...' | base64 -d | sh
```

Paste directly into crontab. The `%` signs are safely inside the base64.

### Exec - Decode And Run Locally

```bash
# Store a command safely, run it later
echo 'complex command with "quotes" and $variables' | entrouter raw-encode > saved.b64
entrouter exec < saved.b64

# Pipe chain
echo 'echo hello world' | entrouter raw-encode | entrouter exec
# hello world
```

Safe command storage in config files, env vars, databases - anywhere that mangles special characters.

### Encode / Decode / Verify

```bash
# Encode anything - comes out as base64 + SHA-256 fingerprint
echo '{"tier":"enterprise","keyType":"engine"}' | entrouter encode
# {"encoded":"eyJ0aWVy...","fingerprint":"3eeb58ed..."}

# Decode it back
echo '{"encoded":"...","fingerprint":"..."}' | entrouter decode
# {"tier":"enterprise","keyType":"engine"}

# Verify it survived the trip
echo '{"encoded":"...","fingerprint":"..."}' | entrouter verify
# INTACT
# Decoded: {"tier":"enterprise","keyType":"engine"}
```

### Raw Mode - Pipe Anywhere

```bash
# Just base64. No JSON wrapper. Perfect for piping.
echo 'hello world' | entrouter raw-encode
# aGVsbG8gd29ybGQ=

echo 'aGVsbG8gd29ybGQ=' | entrouter raw-decode
# hello world

# Encode locally, send over SSH, decode on the other side
echo '{"key":"value"}' | entrouter raw-encode | ssh root@your-vps "entrouter raw-decode"
```

---

## The Library - Six Tools

Entrouter isn't just a CLI. It's a Rust crate with six integrity tools for your backend.

```rust
use entrouter_universal::*;
```

### 1. Envelope - Four Flavours

Wrap your data. Pass it through anything. Unwrap and verify on the other side.

```rust
// Standard - works everywhere
let env = Envelope::wrap(data);
let original = env.unwrap_verified()?;

// URL-Safe - uses - and _ instead of + and /
let env = Envelope::wrap_url_safe(data);

// Compressed - gzip first, then base64 (large payloads)
let env = Envelope::wrap_compressed(data)?;

// TTL - self-expiring (race tokens, sessions, anything time-sensitive)
let env = Envelope::wrap_with_ttl(data, 300); // dies in 5 minutes
env.unwrap_verified() // Err(Expired) after 5 min - cannot be replayed
```

Store it anywhere:
```rust
redis.set("key", env.to_json()?).await?;                                        // Redis
db.execute("INSERT INTO t (envelope) VALUES ($1)", &[&env.to_json()?]).await?;   // Postgres
Response::json(env)                                                               // HTTP (serde-compatible)
```

### 1b. Signed Envelope - HMAC Authentication

Like Envelope, but with HMAC-SHA256 authentication. Proves both **integrity** (SHA-256 fingerprint) AND **origin** (only someone with the key could produce the signature).

```rust
use entrouter_universal::SignedEnvelope;

// Sign with a secret key - four modes just like Envelope
let env = SignedEnvelope::wrap("secret data", "my-secret-key");
let env = SignedEnvelope::wrap_url_safe("secret", "key");
let env = SignedEnvelope::wrap_compressed("large payload", "key")?;
let env = SignedEnvelope::wrap_with_ttl("temporary", "key", 300); // 5 min TTL

// Unwrap - verifies HMAC first, then integrity, then TTL
let original = env.unwrap_verified("my-secret-key")?;

// Wrong key? Rejected before decoding even starts.
assert!(env.unwrap_verified("wrong-key").is_err());
```

Use cases: API tokens, inter-service messages, anything where you need to prove **who** sent it, not just that it arrived intact.

---

### 2. Chain - Cryptographic Audit Trail

Each link references the previous link's fingerprint. Tamper with any link - everything after it breaks. You know exactly where the chain was cut.

```rust
let mut chain = Chain::new("race:listing_abc - OPENED");
chain.append("user_john joined - token: 000001739850000001");
chain.append("user_jane joined - token: 000001739850000002");
chain.append("WINNER: user_john - mathematically proven");
chain.append("race:listing_abc - CLOSED");

assert!(chain.verify().valid);
```

Tamper detection:
```
━━━━ Entrouter Universal Chain Report ━━━━
Links: 5 | Valid: false
  Link 1: ✅ | fp: a3f1b2...
  Link 2: ✅ | fp: 9f8e7d...
  Link 3: ❌ VIOLATED          ← tampered here
  Link 4: ❌ VIOLATED          ← cascade
  Link 5: ❌ VIOLATED          ← cascade
```

**Diff & Merge** — compare two chains, find where they diverge, merge if compatible:

```rust
let diff = Chain::diff(&chain_a, &chain_b);
// ChainDiff { common_length: 3, a_extra: 2, b_extra: 1, diverges_at: Some(4) }

// Merge two chains (one must be a prefix of the other)
let merged = Chain::merge(&chain_a, &chain_b)?; // returns the longer chain
// If they diverge → Err(ChainMergeConflict { diverges_at: 4 })
```

### 3. UniversalStruct - Per-Field Integrity

Not "something broke somewhere." You know **exactly which field** was tampered with.

```rust
let wrapped = UniversalStruct::wrap_fields(&[
    ("token",      "000001739850123456-000004521890000-a3f1b2-user_john"),
    ("user_id",    "john"),
    ("amount",     "299.99"),
    ("listing_id", "listing:abc123"),
]);

assert!(wrapped.verify_all().all_intact);
```

Tamper detection:
```
━━━━ Entrouter Universal Field Report ━━━━
  token:      ✅ - 000001739850123456...
  user_id:    ✅ - john
  amount:     ❌ VIOLATED              ← Redis mangled this one
  listing_id: ✅ - listing:abc123
```

---

### 4. Guardian - Find The Exact Layer That Broke It

Checkpoint your data at every layer. Guardian tells you exactly which one corrupted it.

```rust
let mut g = Guardian::new(data);

g.checkpoint("http_ingress",    &value_at_http);
g.checkpoint("json_parse",      &value_at_json);
g.checkpoint("redis_write",     &value_at_redis);
g.checkpoint("postgres_write",  &value_at_postgres);
```

```
━━━━ Entrouter Universal Pipeline Report ━━━━
  Layer 1: http_ingress   - ✅
  Layer 2: json_parse     - ✅
  Layer 3: redis_write    - ❌ VIOLATED     ← broke here
  Layer 4: postgres_write - ❌ VIOLATED     ← cascade
```

`g.assert_intact()` - panics with the layer name in your tests.

---

### 5. Signed Envelope (HMAC-SHA256)

See [1b. Signed Envelope](#1b-signed-envelope---hmac-authentication) above.

---

### 6. Core Primitives

```rust
use entrouter_universal::{encode_str, decode_str, fingerprint_str, verify};

let encoded  = encode_str(data);          // → base64
let original = decode_str(&encoded)?;     // → original
let fp       = fingerprint_str(data);     // → SHA-256 hex
let intact   = verify(&encoded, &fp)?;    // → bool
```

---

## Why Base64?

| Layer | Characters that break it | Base64 safe? |
|---|---|---|
| HTTP | `"`, `\`, newlines | ✅ |
| JSON | `"`, `\`, control chars | ✅ |
| Shell | `'`, `"`, `$`, `` ` ``, `\`, spaces | ✅ |
| Redis | newlines, spaces | ✅ |
| Postgres | `'`, `\`, null bytes | ✅ |
| URLs | `+`, `/`, `=` (use `wrap_url_safe`) | ✅ |

Every layer sees a boring alphanumeric string. Problem solved at the encoding level - not the escaping level.

---

## Cross-Machine

Base64 and SHA-256 are universal standards. Identical output on Windows, Linux, Mac, ARM, x86, anywhere.

```
Your Machine                         Your VPS
┌────────────────────┐               ┌────────────────────┐
│ Envelope::wrap()   │───── SSH ────►│ Envelope::from_json│
│ or: entrouter ssh  │               │ .unwrap_verified() │
│                    │               │ ✅ Identical.       │
└────────────────────┘               └────────────────────┘
```

---

## Tested Against The Worst

```
SQL injection        ✅  '; DROP TABLE users; --
JSON breaking        ✅  {"key":"val\"ue","nested":"b\\\\c"}
Null bytes           ✅  \x00\x01\x02\x03
Unicode hellscape    ✅  �-�本語 中文 한국어 العربية
Emoji overload       ✅  🔥💀🚀🎯⚡🖤
XSS attempts         ✅  <script>alert('xss')</script>
Redis protocol       ✅  *3\r\n$3\r\nSET\r\n
Path traversal       ✅  ../../../../etc/passwd
Format strings       ✅  %s%s%s%n%n%n
Zero-width chars     ✅  ​‌‍
```

**31 tests. Zero failures.**

---

## Changelog

### v0.8 - Signed Envelopes, Chain Diff/Merge, Multi-Host & Container Execution
- `SignedEnvelope` — HMAC-SHA256 authenticated envelopes (all 4 modes: standard, url-safe, compressed, TTL)
- `Chain::diff()` / `Chain::merge()` — compare and merge cryptographic audit chains
- `entrouter multi-ssh <h1,h2,...>` — fan-out a command to multiple hosts in parallel (CLI + MCP)
- 14 new MCP tools (21 total): envelope wrap/unwrap, signed wrap/unwrap, chain new/append/verify/diff/merge, docker, kube, multi-ssh
- MCP `entrouter_docker` — run commands inside Docker containers via AI agent
- MCP `entrouter_kube` — run commands inside Kubernetes pods via AI agent
- 31 tests passing. New test suites for signed envelopes and chain diff/merge.
- `hmac` 0.12 dependency added

### v0.7 - MCP Server (AI Agent Integration)
- `entrouter mcp` - built-in MCP server for VS Code Copilot, Claude, Cursor, and any MCP-compatible client
- 7 tools exposed: encode, decode, verify, raw-encode, raw-decode, fingerprint, SSH
- `entrouter_ssh` MCP tool - lets AI agents run commands on remote servers with zero escaping
- SSH tool: BatchMode, StrictHostKeyChecking, ConnectTimeout for non-interactive use
- SSH tool: 30-second execution timeout prevents MCP server hangs
- Newline-delimited JSON protocol over stdio

### v0.6 - Docker, Kubernetes, Cron, Exec
- `entrouter docker <container>` - run commands inside Docker containers without escaping
- `entrouter kube <pod> [-n ns]` - run commands inside Kubernetes pods without escaping
- `entrouter cron [schedule]` - encode commands into cron-safe lines (no `%` breakage)
- `entrouter exec` - decode base64 from stdin and execute locally
- Zero dependencies on remote side - uses `base64 -d` for docker/kube/cron
- 11 CLI commands total

### v0.5 - SSH Command
- `entrouter ssh <host>` - type the command, it runs on the remote machine. No escaping.
- Encodes locally, decodes on server, executes via `sh`. One step.

### v0.4 - CLI
- `entrouter` CLI binary - encode, decode, verify, raw-encode, raw-decode from the shell
- Pipe-friendly, works over SSH, no shell escaping issues
- `cargo install entrouter-universal`

### v0.3 - Hardening
- `#[must_use]` on all constructors and pure functions
- `#[non_exhaustive]` on `UniversalError`
- `Clone` and `PartialEq` on all error and result types - testable in assertions
- `Display` impls on `VerifyResult`, `ChainVerifyResult`, `StructVerifyResult`, `FieldVerifyResult`
- New `SerializationError` variant - JSON failures no longer misreport as `MalformedEnvelope`
- Guardian now reports decode failures instead of silently swallowing them
- 16 MiB decompression size guard against gzip bombs

---

## License
Apache-2.0 - Free for open-source.
Commercial license available for closed-source / proprietary use.
Contact hello@entrouter.com

*Part of the Entrouter suite - [entrouter.com](https://entrouter.com)*