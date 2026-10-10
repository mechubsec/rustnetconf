<!-- mechub-version: v0.18.1 -->

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/assets/mechub-mark.svg">
    <img src="docs/assets/mechub-mark-light.svg" width="72" alt="mechub mark">
  </picture>
</p>

<h1 align="center">rustnetconf</h1>

<p align="center"><strong>A Rust network automation platform</strong><br>
<em>a mechub project — sovereign network-security automation</em></p>

<p align="center">
  <a href="https://crates.io/crates/rustnetconf"><img alt="crates.io — rustnetconf" src="https://img.shields.io/crates/v/rustnetconf.svg?label=rustnetconf&color=0D9488"></a>
  <a href="https://crates.io/crates/rustnetconf-cli"><img alt="crates.io — rustnetconf-cli" src="https://img.shields.io/crates/v/rustnetconf-cli.svg?label=rustnetconf-cli&color=262B38"></a>
  <a href="https://crates.io/crates/rustnetconf-yang"><img alt="crates.io — rustnetconf-yang" src="https://img.shields.io/crates/v/rustnetconf-yang.svg?label=rustnetconf-yang&color=262B38"></a>
  <a href="https://github.com/mechubsec/rustnetconf/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/mechubsec/rustnetconf/actions/workflows/ci.yml/badge.svg"></a>
  <a href="#license"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-262B38.svg"></a>
</p>

> **Unofficial / community project.** This repository is an independent, community-driven project. It is not affiliated with, endorsed by, sponsored by, or supported by Hewlett Packard Enterprise or Juniper Networks. "HPE", "Juniper", "SRX", "JUNOS", "Security Director" and "Juniper Mist" are trademarks of their respective owners and are used here only to describe what this software interoperates with. Please direct support and licensing questions about those products to the respective vendors.

Async NETCONF client library, YANG code generation, vendor profiles, connection pooling, and a Terraform-like CLI for declarative network config management.

Built on [tokio](https://tokio.rs), [russh](https://crates.io/crates/russh), and [rustls](https://crates.io/crates/rustls) — pure Rust, no OpenSSL, no libssh2.

> **Latest release — [v0.18.1](https://github.com/mechubsec/rustnetconf/releases/tag/v0.18.1)** (security: russh bumped to fix CVE-2026-102823).
> On crates.io: `rustnetconf` 0.18.1 · `rustnetconf-cli` 0.5.0 · `rustnetconf-yang` 0.5.0.
> See [CHANGELOG.md](CHANGELOG.md#whats-new-in-v0181) for release notes.

## Install

Requires **Rust 1.89+**. `rustup` installs the toolchain pinned in
`rust-toolchain.toml` automatically; distro-packaged `cargo` is often older
than 1.89 and will fail on dependency rustc requirements.

```bash
cargo install --locked rustnetconf-cli
```

This installs the package `rustnetconf-cli`, but the resulting executable is
named **`netconf`** — that's the binary invoked throughout this README.

Building from source instead:

```bash
cargo build --release -p rustnetconf-cli   # -> target/release/netconf
```

## Workspace

| Crate | Description |
|-------|-------------|
| **rustnetconf** | Async NETCONF 1.0/1.1 client library |
| **rustnetconf-yang** | YANG model code generation (compile-time config validation) |
| **rustnetconf-cli** | Terraform-like CLI tool (`netconf` binary) |

## Scope

This is a **NETCONF library** — RFC 6241 (the protocol) and RFC 6242 (NETCONF over SSH) — plus YANG code generation and a CLI built on it.

SSH is present as a *transport for NETCONF*, not as a general-purpose capability. The crate deliberately does **not** provide:

- File transfer (SFTP, SCP) — it is not a file-transfer library
- A public `russh` handle, or a generic "open any SSH subsystem" escape hatch — the SSH connection is an implementation detail of the NETCONF transport, and keeping it private is what stops this crate's public API from becoming russh's version surface
- Remote shell or command execution

Consumers that need those should use a dedicated SSH crate alongside this one. A native SCP1 client was briefly added and then reverted before it was ever released (#52, reverted by #53) for exactly this reason; issues #47 and #51 were closed as not planned on the same grounds. The round trip is visible in `git log` between v0.13.2 and the next release — it was a deliberate reversal, not an accident.

## Release History

Full release notes, including breaking-change migration guidance for every
past version, live in [CHANGELOG.md](CHANGELOG.md).

## RFC Support

| RFC | Feature | Status |
|-----|---------|--------|
| RFC 6241 | Network Configuration Protocol (NETCONF) | ✅ supported |
| RFC 6242 | NETCONF over SSH | ✅ supported |
| RFC 7589 | NETCONF over TLS | ✅ supported (feature flag `tls`) — **needs physical SRX or non-vSRX for TLS test** |
| RFC 5277 | Event Notifications | ✅ supported — tested on Junos 24.4 vSRX (subscription + capability; interleave limited by device) |
| RFC 5717 | Partial Lock RPC | ✅ supported — gated on `:partial-lock:1.0` |
| RFC 8071 | NETCONF Call Home | 💡 planned |
| RFC 6243 | With-defaults Capability | ✅ supported — mode gated on the device's `basic-mode`/`also-supported` list |
| RFC 6022 | YANG Module for NETCONF Monitoring | 💡 planned |
| RFC 8526 | NETCONF Extensions for NMDA | 💡 planned |
| RFC 6470 | NETCONF Base Notifications | 💡 planned |
| RFC 8040 | RESTCONF | 💡 planned |

## CLI Tool — `netconf`

Declarative network config management. Write desired state as XML files, the CLI diffs against the device and applies changes with confirmed-commit safety.

```bash
netconf init                    # Create project skeleton
netconf plan spine-01           # Show what would change (colored diff)
netconf apply spine-01          # Apply with confirmed-commit (auto-revert on timeout)
netconf confirm spine-01        # Make changes permanent
netconf rollback spine-01       # Revert to saved state
netconf get spine-01            # Fetch running config
netconf validate spine-01       # Dry-run validation
```

### Project Structure

```
my-network/
├── inventory.toml              # Device connection details
├── desired/
│   └── spine-01/
│       ├── interfaces.xml      # Desired interface config
│       └── system.xml          # Desired system config
└── .netconf/state/             # Rollback snapshots (auto-managed)
```

### inventory.toml

```toml
[defaults]
confirm_timeout = 60

[devices.spine-01]
host = "10.0.0.1:830"
username = "admin"
key_file = "~/.ssh/id_ed25519"
# vendor auto-detected from device hello
```

**Secrets:** `inventory.toml` may contain plaintext passwords. Prefer
`key_file` or SSH-agent auth where possible. If you must use inline
passwords, protect the file with `chmod 600 inventory.toml` and add it
to `.gitignore`. Passwords are stored in zeroizing memory and redacted
from `Debug` output, but the on-disk file itself is plaintext.

## Library — Quick Start

```toml
[dependencies]
rustnetconf = { git = "https://github.com/mechubsec/rustnetconf.git" }
tokio = { version = "1", features = ["full"] }
```

For TLS transport (RFC 7589), enable the `tls` feature:

```toml
[dependencies]
rustnetconf = { git = "https://github.com/mechubsec/rustnetconf.git", features = ["tls"] }
```

### Fetch running config

```rust
use rustnetconf::{Client, Datastore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::connect("10.0.0.1:830")
        .username("admin")
        .key_file("~/.ssh/id_ed25519")
        .connect()
        .await?;

    let config = client.get_config(Datastore::Running).await?;
    println!("{config}");

    client.close_session().await?;
    Ok(())
}
```

### Edit config (full round trip)

```rust
use rustnetconf::{Client, Datastore, DefaultOperation};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::connect("10.0.0.1:830")
        .username("admin")
        .password("secret")
        .connect()
        .await?;

    client.lock(Datastore::Candidate).await?;

    client.edit_config(Datastore::Candidate)
        .config("<interface><name>ge-0/0/0</name><description>uplink</description></interface>")
        .default_operation(DefaultOperation::Merge)
        .send()
        .await?;

    client.validate(Datastore::Candidate).await?;
    client.commit().await?;
    client.unlock(Datastore::Candidate).await?;

    client.close_session().await?;
    Ok(())
}
```

### Connect through a jump host (`ProxyJump`)

```rust
use rustnetconf::{Client, Datastore};
use rustnetconf::transport::ssh::{JumpHostConfig, SshAuth, HostKeyVerification};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bastion = JumpHostConfig {
        host: "bastion.example.com".into(),
        port: 22,
        username: "jumpuser".into(),
        auth: SshAuth::Agent,
        host_key_verification: HostKeyVerification::AcceptAll,
    };

    let mut client = Client::connect("10.0.0.1:830")
        .username("admin")
        .ssh_agent()
        .jump_hosts(vec![bastion])
        .connect()
        .await?;

    let config = client.get_config(Datastore::Running).await?;
    println!("{config}");
    client.close_session().await?;
    Ok(())
}
```

### Connect using your `~/.ssh/config`

```rust
use rustnetconf::{Client, Datastore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Resolves `Host edge-r1` from ~/.ssh/config — picks up HostName, Port,
    // User, IdentityFile, ProxyJump, ProxyCommand. NETCONF default port 830
    // is used when the config doesn't pin Port.
    let mut client = Client::connect_via_ssh_config("edge-r1")?
        .ssh_agent()
        .connect()
        .await?;

    let config = client.get_config(Datastore::Running).await?;
    println!("{config}");
    client.close_session().await?;
    Ok(())
}
```

### Connect over TLS (RFC 7589)

> **Note:** vSRX 24.4 has a known TLS handshake issue where the PKI engine cannot
> present a self-signed certificate chain. TLS testing requires a physical SRX,
> MX, or EX device with a CA-signed certificate. The code compiles and passes
> unit tests but has not been validated against a live TLS-capable device.

```rust
use rustnetconf::{Client, TlsConfig, Datastore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = TlsConfig {
        host: "10.0.0.1".into(),
        ca_cert: Some("ca.pem".into()),
        client_cert: Some("client.pem".into()),
        client_key: Some("client-key.pem".into()),
        ..Default::default()
    };

    let mut client = Client::connect_tls(config).connect().await?;
    let config = client.get_config(Datastore::Running).await?;
    println!("{config}");

    client.close_session().await?;
    Ok(())
}
```

### Event notifications (RFC 5277)

```rust
use rustnetconf::{Client, Datastore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::connect("10.0.0.1:830")
        .username("admin")
        .password("secret")
        .connect()
        .await?;

    // Subscribe to NETCONF event stream
    client.create_subscription(Some("NETCONF"), None, None, None).await?;

    // Block waiting for notifications
    while let Some(notif) = client.recv_notification().await? {
        println!("[{}] {}", notif.event_time, notif.event_xml);
    }

    Ok(())
}
```

> **Note:** Some devices (e.g., Junos vSRX 24.4) advertise `:interleave` but do not
> respond to RPCs on a session with an active subscription. On these devices, use a
> dedicated session for notifications and a separate session for RPCs. Notifications
> arriving during RPCs on interleave-capable devices are automatically buffered and
> available via `drain_notifications()`.

### Connection pooling

```rust
use rustnetconf::pool::{DevicePool, DeviceConfig};
use rustnetconf::transport::ssh::SshAuth;
use rustnetconf::Datastore;
use zeroize::Zeroizing;

let pool = DevicePool::builder()
    .max_connections(50)
    .add_device("spine-01", DeviceConfig {
        host: "10.0.0.1:830".into(),
        username: "admin".into(),
        auth: SshAuth::KeyFile { path: "~/.ssh/id_ed25519".into(), passphrase: None },
        vendor: None, // auto-detect
    })
    .build();

let mut conn = pool.checkout("spine-01").await?;
let config = conn.get_config(Datastore::Running).await?;
// connection auto-returned to pool on drop
```

## Features

### NETCONF Client
- **Async-first** — tokio-based, push config to 500 devices concurrently
- **SSH + TLS transports** — SSH (RFC 6242) by default, TLS (RFC 7589) via `tls` feature flag
- **SSH bastion support** — `ProxyJump` (multi-hop), `ProxyCommand` (shell-escaped), and OpenSSH `~/.ssh/config` alias resolution
- **NETCONF 1.0 + 1.1** — EOM and chunked framing with auto-negotiation
- **All core RPCs** — get, get-config, edit-config, copy-config, delete-config, lock/unlock, commit, confirmed-commit, cancel-commit, validate, close/kill-session, discard-changes
- **With-defaults (RFC 6243)** — `report-all`, `report-all-tagged`, `trim`, `explicit`, on get, get-config and copy-config, each gated on the device's advertised `basic-mode`/`also-supported` list. Without this, an absent leaf and a leaf sitting at its YANG default are indistinguishable, so a diff shows phantom changes on devices that report defaults and misses them on devices that don't
- **Subtree and XPath filters** — `SubtreeFilter` builds subtree filters; `XPathFilter` builds XPath ones (RFC 6241 §6.4) with namespace binding, gated on the device advertising `:xpath:1.0` so an unsupported filter fails loudly instead of silently returning the whole datastore
- **Confirmed commit** — auto-rollback safety net (RFC 6241 §8.4)
- **Event notifications** — `create-subscription`, inline notification demux, buffered drain/recv API (RFC 5277)
- **Streaming replies** — `get_config_streaming` / `get_streaming` write the
  reply into any `AsyncWrite` as it arrives, so peak memory is one read plus
  one frame chunk instead of the whole response. Lets you fetch a config
  larger than the read ceiling, which the buffered call cannot do at any
  setting. The caller gets the raw `<rpc-reply>` — vendor unwrapping and
  reply repair both need the complete document, so neither runs on a stream
- **RPC timeout** — configurable per-session deadline prevents indefinite blocking on unresponsive devices
- **XML fragment validation** — user-provided RPC content is validated before insertion to prevent XML injection
- **CommitUnknown detection** — distinguishes "commit failed" from "maybe committed, connection lost"
- **Stale lock recovery** — `lock_or_kill_stale()` kills crashed sessions holding locks
- **Partial locking (RFC 5717)** — lock only the subtrees an XPath names, **in `running`**. RFC 5717 does not cover `candidate` or `startup`, so this is not a substitute for the candidate lock the `netconf apply` flow takes. An indeterminate result poisons the session so a pooled connection is never recycled holding a lock it cannot release; a definitive rejection such as `lock-denied` does not, since a failed partial lock is atomic
- **Framing mismatch detection** — catches firmware bugs where devices send wrong framing
- **IPv6 support** — connect to devices using bracket notation (`[::1]:830`) or bare IPv6 addresses

### Vendor Profiles
- **Auto-detection** from device `<hello>` capabilities
- **Junos** — config wrapping, namespace normalization, discard-before-close
- **Generic** — standard RFC 6241 for any compliant device
- Extensible — implement `VendorProfile` trait for custom vendors

### Connection Pool
- Tokio semaphore-based concurrency limiting
- Checkout with timeout (no blocking forever)
- Auto-checkin on drop with health check — dead connections are discarded, not recycled
- Connection reuse from idle pool

### YANG Code Generation
- Pregenerated Rust types committed at `rustnetconf-yang/src/generated.rs` — no libyang2 or cmake needed to build or use the crate
- Regeneration via `cargo run -p rustnetconf-yang --features regenerate --bin codegen` when `yang-models/` changes — libyang2 and cmake required only then
- Typed Rust structs with serde Serialize/Deserialize
- Full XML serialization — leaves, containers, and lists
- Correct type mapping (string, bool, uint32, etc.)
- Complete Rust keyword escaping for YANG node names
- Bundled IETF models: ietf-interfaces, ietf-ip, ietf-yang-types, ietf-inet-types

### Authentication
| Method | Transport | Builder API |
|--------|-----------|-------------|
| Password | SSH | `.password("secret")` |
| Key file | SSH | `.key_file("~/.ssh/id_ed25519")` |
| SSH agent | SSH | `.ssh_agent()` |
| Server-only TLS | TLS | `TlsConfig { ca_cert, .. }` |
| Mutual TLS (mTLS) | TLS | `TlsConfig { client_cert, client_key, .. }` |

### SSH Connection Options
| Option | Builder API | Notes |
|--------|-------------|-------|
| Direct TCP | (default) | No proxy |
| `ProxyJump` (bastion chain) | `.jump_hosts(Vec<JumpHostConfig>)` | Each hop has its own credentials and host-key policy |
| `ProxyCommand` | `.proxy_command("ssh -W %h:%p bastion")` | `%h`/`%p` shell-escaped and substituted; runs under `sh -c` |
| `~/.ssh/config` alias | `Client::connect_via_ssh_config("alias")?` | Resolves `HostName`, `Port`, `User`, `IdentityFile`, `ProxyJump`, `ProxyCommand`, `Include` |

`jump_hosts` and `proxy_command` are mutually exclusive at connect time.

### Error Handling

Layered errors matching the protocol stack:

```rust
match result {
    Err(NetconfError::Transport(e)) => { /* SSH/TLS connection issues */ }
    Err(NetconfError::Framing(e))   => { /* Protocol framing errors */ }
    Err(NetconfError::Rpc(e))       => { /* Device rejected RPC (all 7 RFC fields parsed) */ }
    Err(NetconfError::Protocol(e))  => { /* Capability/session errors */ }
    Ok(response) => { /* Success */ }
}
```

## Supported Operations

| Operation | RFC 6241 | Status |
|-----------|----------|--------|
| `get` | §7.7 | Done |
| `get-config` | §7.1 | Done |
| `edit-config` | §7.2 | Done |
| `copy-config` | §7.3 | Done |
| `delete-config` | §7.4 | Done (startup / url) |
| `lock` / `unlock` | §7.5-7.6 | Done |
| `close-session` | §7.8 | Done |
| `kill-session` | §7.9 | Done |
| `commit` | §8.4 | Done |
| `confirmed-commit` | §8.4 | Done |
| `cancel-commit` | §8.4.4.1 | Done |
| `partial-lock` / `partial-unlock` | RFC 5717 | Done |
| `validate` | §8.6 | Done |
| `discard-changes` | §8.3 | Done |

## Testing

`cargo test --workspace --locked` runs 654 passed / 0 failed / 7 ignored
(674 with `--all-features`, the count CI runs):
- **Unit tests** — framing, RPC serialization, capability parsing, vendor profiles, diff engine, inventory parsing, IPv6 address parsing, XML fragment validation, capability normalization
- **Mock transport tests** — session state machine, CommitUnknown detection, lock recovery
- **Integration tests** — against a live Juniper vSRX including full edit-config round trips, vendor auto-detection, connection pooling, and concurrent sessions; opt-in, see below

### Prerequisites

None for building, testing, or using this crate. `rustnetconf-yang` ships
with pregenerated Rust types committed at
`rustnetconf-yang/src/generated.rs`, so consumers link no libyang and need
no cmake:

```bash
cargo tree -p rustnetconf-yang --features generated -i libyang2-sys   # finds nothing
cargo test --workspace                                                 # works without cmake
```

**cmake and libyang2 are needed only to regenerate** — when `yang-models/`
changes, or when maintaining the crate itself. The generator is now a bin
target gated on the `regenerate` feature:

```bash
cargo run -p rustnetconf-yang --features regenerate --bin codegen
```

This rewrites `rustnetconf-yang/src/generated.rs` and runs rustfmt on it. CI
has a `yang-codegen-drift` job that re-runs it and fails on any diff, so a
stale committed file cannot merge.

**Install cmake only if you need to regenerate:**

```bash
# Debian/Ubuntu
sudo apt-get install cmake

# macOS
brew install cmake

# Fedora/RHEL
sudo dnf install cmake
```

The `bundled` feature, still in `default`, now compiles libyang2 from source
when regenerating. It is inert unless `regenerate` is also enabled. To use a
system libyang2 instead (~44 MB of build artifacts saved), opt out:

```bash
cargo run -p rustnetconf-yang --no-default-features --features regenerate --bin codegen
```

**This path needs a libyang providing `libyang.so.3`** — SONAME 3, not
SONAME 2. The two are binary-incompatible.

The reason still matters, even though the risk is now maintainer-only:
`libyang2-sys` probes for `libyang` with no version constraint and falls
back to a bare `-lyang`, while the bindings it ships are generated against
SONAME 3. An ABI-2 library would therefore link without complaint and then
feed the generator wrong struct offsets while it walks libyang's C types —
silent undefined behaviour during code generation rather than a build
failure.

**We do not verify this for you, and the build says so.** Checking it
properly means knowing which file `-lyang` actually resolves to, which
depends on `-L` ordering, symlink targets, platform library naming, and —
because the generator is a host binary — on host rather than target settings
when cross-compiling. libyang exposes no runtime soversion accessor through
these bindings to settle it. A check that guessed and reported "OK" would be
worse than none, so the build emits a warning stating the ABI was not
verified and leaves the responsibility with you.

Check the **soname**, not the package version:

```bash
ls $(pkg-config --variable=libdir libyang)/libyang.so.*   # want libyang.so.3
```

`pkg-config --modversion libyang` is *not* the right check. libyang carries
a project version and a soversion that deliberately disagree — the release
vendored here is `LIBYANG_VERSION 2.2.8` with `LIBYANG_MAJOR_SOVERSION 3`,
and `libyang.pc` publishes the former. Gating on the package version rejects
exactly the library you want.

Without any system libyang on the linker path, the build fails with `unable
to find library -lyang`. That and the explicit ABI error are both expected
outcomes — install a libyang providing SONAME 3, or stay on `bundled`.

```bash
cargo test --workspace                           # Run all tests (no cmake needed)
cargo test --test integration_vsrx               # Run vSRX integration tests only
```

Live-device integration tests are opt-in, not opt-out: `tests/integration_vsrx.rs`
early-returns unless `RUSTNETCONF_TEST_VSRX_HOST` is set, so a plain
`cargo test --workspace` already skips them and needs no device. Set
`RUSTNETCONF_TEST_VSRX_REQUIRED=1` when a run is meant to exercise a real
device — it turns every skip into a failure that names its reason, so a
misspelled variable or an unreadable key file fails the run instead of
quietly passing it.

## Security

### Known Issues

- **Debug logs may contain file paths** — When SSH key file loading fails, the key file path is included in `tracing::debug!` output. This is not exposed at info/warn/error levels. **Mitigation:** Disable debug-level logging in production, or filter `rustnetconf::transport` logs.

### Security Features

- **Credential zeroization** — Passwords and key passphrases use `Zeroizing<String>` (via the `zeroize` crate) and are securely erased from memory on drop.
- **SSH host key verification** — `HostKeyVerification` must be set explicitly. The `ClientBuilder` default is `RejectAll` (fail closed): the SSH handshake fails until the caller pins a fingerprint via `Fingerprint("SHA256:...")` or explicitly opts in to `AcceptAll` for lab use (logs a `tracing::warn!`). `ProxyJump` hops parsed from `~/.ssh/config` likewise default to `RejectAll` and must be individually configured. In the CLI, set `host_key_fingerprint` per device in `inventory.toml`, or pass `--insecure-accept-host-key` for lab use only.
- **Shell-escaped ProxyCommand** — `%h` and `%p` substitutions are shell-escaped to prevent command injection via malicious hostnames.
- **XML fragment validation** — All user-provided RPC content is validated before insertion. Beyond well-formedness, a fragment must be *balanced* and must not close an element it did not open: a trailing bare `<` otherwise consumes the enclosing element's closing tag, and a fragment naming the validator's own synthetic root can balance against it. Both were found by fuzzing (`fuzz/README.md`) rather than by inspection.
- **Strict XML validation (opt-in)** — the `strict-validation` feature adds
  `validate_xml_fragment_strict`, which checks a fragment by attempting a
  *conforming* parse of the embedded document rather than by a hand-written rule
  list. Fuzzing found fourteen escapes from the rule list, and further rounds
  kept finding more after it was believed complete (`fuzz/README.md` has the
  table). Off by default: it costs one small pure-Rust dependency
  (142 → 143 crates), which is the caller's trade to make
- **XML attribute escaping** — All message-id values are escaped to prevent XML attribute injection.
- **TLS bypass warnings** — `danger_accept_invalid_certs` emits a detailed warning explaining that ALL certificate validation is bypassed (trust chain, signatures, hostname, and expiry).
- **Read buffer limits** — Session read buffers default to 100 MB (configurable via `.max_read_buffer()`) to prevent memory exhaustion.
- **RPC timeout** — Configurable via `.rpc_timeout()` to prevent indefinite blocking on unresponsive devices.
- **CLI input validation** — Device names are validated to prevent path traversal; state files are written with `0600` permissions on Unix.
- **Typed error hierarchy** — Structured error types (`ChannelClosed`, `SessionExpired`, `MessageIdMismatch`) enable precise error handling without string matching.
- **No unsafe code, enforced across every target** — declared as `unsafe_code = "forbid"` in `[workspace.lints.rust]`, which each package opts into. This is deliberate rather than a crate-root `#![forbid(unsafe_code)]`: build scripts, examples and integration tests compile as separate crates, so an inner attribute would not reach them. Verified by injecting an `unsafe` block into `build.rs`, a test target and an example, each of which now fails to compile. The one place that previously needed `unsafe` was two tests setting `HOME` via `std::env::set_var`, which the 2024 edition makes unsafe; the tilde expansion they covered was split into a pure function taking the home directory as an argument.

### Known advisories

None currently suppressed — `.cargo/audit.toml` has an empty `ignore` list,
and `cargo audit` plus `cargo deny` run on every CI build.

**RUSTSEC-2023-0071** (Marvin Attack, `rsa` timing side-channel) was carried
here until russh 0.62. It is now *absent from the graph*, not tolerated within
it: russh gates `rsa` behind a non-default feature and this crate builds with
`default-features = false`, so both of these come up empty:

```bash
grep '^name = "rsa"' Cargo.lock
cargo tree --workspace --all-features -i rsa
```

The suppression was removed rather than left in place, so that a future
dependency bump reintroducing `rsa` fails the audit loudly instead of passing
under an ignore nobody re-reads.

### Security Best Practices

- Use Ed25519 SSH keys (not RSA) for device authentication — smaller, faster,
  constant-time by construction, and not exposed to the RSA timing
  side-channel class of bug in the first place
- Set `host_key_verification(HostKeyVerification::Fingerprint(...))` or `HostKeyVerification::KnownHosts(path)` in production — the default is `RejectAll` (fail closed), so the connection will refuse to complete until you choose a policy. For the CLI, set either `host_key_fingerprint = "SHA256:..."` or `known_hosts_path = "/path/to/known_hosts"` per device in `inventory.toml` (or `known_hosts_path` under `[defaults]` for fleet-wide pinning). See `examples/known_hosts.rs` for the `ssh-keyscan` workflow.
- Set `.rpc_timeout(Duration::from_secs(30))` to prevent hanging on unresponsive devices
- Prefer SSH agent auth over inline passwords
- Store credentials in inventory.toml with restricted file permissions (`chmod 600`)
- Run the CLI on trusted management networks with direct device connectivity
- Use `confirmed-commit` (the default for `netconf apply`) so the device auto-reverts if something goes wrong
- Disable debug-level logging in production environments

To report a security vulnerability, please open an issue on GitHub.

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `async-trait` | 0.1 | Async trait support |
| `aws-lc-rs` | 1 | SHA-256 fingerprints and HMAC for `known_hosts` |
| `base64ct` | 1 | Constant-time base64 for key blobs |
| `quick-xml` | 0.41 | XML parsing (NETCONF RPC encode/decode) |
| `russh` | 0.62 | SSH transport (pure Rust, no libssh2) |
| `thiserror` | 2 | Error derive macros |
| `tokio` | 1 | Async runtime |
| `tracing` | 0.1 | Structured logging/tracing |
| `zeroize` | 1 | Secure credential erasure on drop |

Optional (behind `tls` feature):

| Crate | Version | Purpose |
|-------|---------|---------|
| `rustls` | 0.23 | TLS transport (pure Rust, no OpenSSL) |
| `tokio-rustls` | 0.26 | Async TLS stream adapter |
| `webpki-roots` | 0.26 | Mozilla CA root certificates |

Dev-only:

| Crate | Version | Purpose |
|-------|---------|---------|
| `tokio-test` | 0.4 | Async test utilities |
| `tracing-subscriber` | 0.3 | Log subscriber for tests |
| `tempfile` | 3 | Temporary directories for tests |

## License

MIT

## Contributing

Contributions welcome! See [ARCHITECTURE.md](ARCHITECTURE.md) for the codebase design and [TODOS.md](TODOS.md) for tracked work items.

---

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/assets/mechub-mark.svg">
    <img src="docs/assets/mechub-mark-light.svg" width="28" alt="">
  </picture><br>
  <sub><code>a mechub project</code> · deterministic decides · the model explains · a human approves<br>
  <a href="https://github.com/fastrevmd-lab">github.com/fastrevmd-lab</a></sub>
</p>
