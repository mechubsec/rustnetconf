# Changelog

Release notes for `rustnetconf`, `rustnetconf-cli`, and `rustnetconf-yang`. See the [README](README.md) for current usage.

## What's New in v0.18.1

**Security: russh bumped to `>=0.63.2, <0.64`** (#121, CVE-2026-102823).

rustnetconf pinned russh 0.62, so every consumer's lockfile still carried the vulnerable 0.62.x line transitively even after rustjunosmcp and mecmcp moved their own direct dependencies to 0.63.x. `check_server_key` adapts to the renamed trait signature: russh 0.63 generalized the host-key callback from `&PublicKey` to `&PublicKeyOrCertificate` so client certificate auth can share the same verification path. Fingerprinting and known-hosts/pin checks now run against `PublicKeyOrCertificate::public_key()`, which resolves to the same `PublicKey` as before for the non-certificate case this crate uses today. No public API change.

### `rustnetconf-cli` and `rustnetconf-yang`

Both stay at 0.5.0 — neither has a code change of its own in this release — but their `rustnetconf` dependency requirement moves to `0.18.1` so a fresh `cargo publish`/`cargo install` pulls in the fixed russh line rather than resolving to 0.18.0.

## What's New in v0.18.0

**`HostKeyVerification` gains `AcceptNew`, a real trust-on-first-use mode** (#108, MEC-43).

Previously `HostKeyVerification` had no TOFU mode: `KnownHosts` fails closed on an unknown host, and `AcceptAll` pins nothing at all. Consumers that wanted "accept new hosts, but still pin them" — rustjunosmcp's "accept new host keys" flag among them — had nothing better to map to than `AcceptAll`, silently giving up pinning entirely.

`AcceptNew { known_hosts: PathBuf }` follows OpenSSH's `StrictHostKeyChecking=accept-new`: an unknown host is accepted and its key appended to the known-hosts file (created at mode `0600` if absent, in the plain `[host]:port keytype base64` form), while a known host presenting a different key still rejects with `HostKeyMismatch`, and `@revoked` entries still reject — identical behavior to `KnownHosts` for hosts already on file. Every first-contact pin logs a `warn` with the host and fingerprint.

### Also in this release

- **`rustnetconf-yang`'s generated code is now committed, and `libyang2` is maintainer-only** (#105, #106). The YANG codegen build script previously ran unconditionally, charging every build a hard `cmake` prerequisite and ~44 MB of vendored C per build directory to regenerate 441 lines of Rust that only change when `yang-models/` changes. That output is now committed (`rustnetconf-yang/src/generated.rs`); the generator moves to a `regenerate`-feature-gated `codegen` bin target. Ordinary consumers of `rustnetconf-yang` no longer need `cmake` or `libyang2` at all — only someone regenerating the bindings does. CI gained a `yang-codegen-drift` job so a stale `generated.rs` can't merge.
- **`fix(deps)`: rustls bumped to 0.23.45** for RUSTSEC-2026-0285 (#111).
- Hygiene: extended the shared gitleaks vendor allowlist (#110), replaced lab identifiers with synthetic values across fixtures and docs (#112), and added `CONTRIBUTING.md`, issue/PR templates, and `SECURITY.md` as a release baseline (#109). No functional change in any of these.

### `rustnetconf-cli` 0.5.0 and `rustnetconf-yang` 0.5.0

Both move only because they require `rustnetconf = "0.18.0"` exactly — neither has a functional change of its own in this release. Pairing either with `rustnetconf` 0.17.x is not supported.

## What's New in v0.17.0

**The SSH transport moves from `aws-lc-rs` to `ring`; TLS deliberately does not** (#102, part of #77).

The default build now links no aws-lc at all:

```
$ cargo tree -i aws-lc-rs
error: package ID specification `aws-lc-rs` did not match any packages
```

| build | size |
|---|---|
| before (aws-lc) | 9,816,520 B |
| after (ring) | 7,505,472 B |
| delta | **−2,311,048 B, 23.5% smaller** |

Build artifacts on disk drop from 240 MB of `aws-lc-sys` build directories to ring's 21 MB. The crate count barely moves, 140 → 139 — worth stating plainly, since #77 opened on dependency count: the win is vendored C and binary size, not crate count.

**TLS keeps aws-lc, on purpose.** Both providers were enumerated rather than assumed:

```
ring    kx groups: X25519, secp256r1, secp384r1
aws-lc  kx groups: X25519, secp256r1, secp384r1, X25519MLKEM768
ring    verify:    no ECDSA_NISTP521_SHA512
aws-lc  verify:    ECDSA_NISTP521_SHA512
```

Moving TLS to ring would silently drop hybrid post-quantum key exchange and P-521 certificate verification. `tls` is a non-default feature, so the ordinary build is already ring-only and gets the whole reduction; only a caller opting into NETCONF-over-TLS links aws-lc, and they get those capabilities for it.

**Why this is 0.17.0 and not 0.16.3.** No public signature changes. But which cryptographic provider a consumer links, and therefore which algorithms are available to the SSH transport, is not a patch-level detail — a consumer that depended on aws-lc being present in the default graph will no longer find it.

### Interop

Exercised against vSRX 24.4R1.9. Not yet run end-to-end against the lab's SRX345 at 21.2R3-S6.11, which #77 names as the interop condition, so **#77 stays open**. What was measured is that 21.2's SSH algorithm offer is a strict superset of 24.4's — host keys, ciphers and MACs byte-identical, plus group16/group18 — so the intersection ring already negotiates against 24.4 is present in full on 21.2. Evidence, not closure; recorded on #77.

### Also in this release

- **The rustls provider choice is now actually ours** (#100). `tokio-rustls` was left at default features, and tokio-rustls 0.26's `default` includes `aws_lc_rs`, forwarding to `rustls/aws_lc_rs`. That silently overrode the `default-features = false` on the rustls line directly above it. Nothing differed at the time — both named the same backend — which is exactly why it was worth fixing before it bit: a later attempt to change the provider on the rustls line alone would have been ignored. Same hazard as rustpanosmcp#149: what a manifest asks for is not what the graph resolves to.
- **`argon2` and `blake2` are no longer release candidates** (#101, part of #64). Both reached stable, and `ssh-key`'s requirements were already loose enough to accept them; they were pinned to RCs in the lockfile only. Prerelease crates in the tree: 3 → 1. **`ssh-key` remains at `0.7.0-rc.11`**, so #64 stays open on that one crate.
- **The CLI drops `colored`** (#99, #77 action 5). Its fourteen call sites in `diff/format.rs` moved to `console::style`, already in the tree via dialoguer — three styling stacks become two. Total crate count is unchanged at 243; the win is one fewer styling API to maintain, not bytes.

### `rustnetconf-cli` 0.4.0 and `rustnetconf-yang` 0.4.0

Both move only because they require `rustnetconf = "0.17.0"` exactly. `rustnetconf-yang` has no functional change in this release; `rustnetconf-cli`'s only change is the styling stack above. Pairing either with `rustnetconf` 0.16.x is not supported.

## What's New in v0.16.2

The quick-xml 0.42 migration (#70). 0.42 decodes at the reader, so events hand back `str` rather than `[u8]`, which removes most of the decode / `from_utf8_lossy` layer this crate carried. The change is mostly deletion: 239 insertions against 277 deletions.

**`rustnetconf` 0.16.2 is a patch — quick-xml is fully internal there.** No public signature, error variant, or trait exposes it, verified by reading every public item.

**`rustnetconf-yang` 0.3.0 is a breaking change, and this is the one to read.** Unlike the library crate, yang puts quick-xml in its *public* API:

```rust
pub use quick_xml::Writer;

pub trait WriteXmlFields {
    fn write_xml_fields(&self, writer: &mut Writer<Cursor<Vec<u8>>>) -> Result<(), XmlError>;
}
```

`new_writer`, `finish_writer`, `write_text_element`, `write_start_with_ns`, `write_end` and `write_element_with_fields` all take or return that type. A `Writer` from quick-xml 0.41 and one from 0.42 are distinct types from distinct crate versions, so any code that implements `WriteXmlFields` by hand, or that holds a `Writer` it obtained elsewhere, needs to move to 0.42 at the same time. Code that only uses generated types and calls `to_xml()` is unaffected.

`rustnetconf-cli` 0.3.9 is a patch: its `diff/tree.rs` moved to the 0.42 event API, and its entity resolution still uses `resolve_xml_entity` — the `escape-html` feature-unification hazard fixed in 0.16.1 is unchanged by the version bump.

Both `rustnetconf-cli` and `rustnetconf-yang` now require `rustnetconf = "0.16.2"` rather than a looser range. quick-xml 0.41 and 0.42 are semver-incompatible, so a graph that paired the new CLI or yang with `rustnetconf` 0.16.1 would build **both** copies of quick-xml.

### What the migration decided, where a choice existed

- **Kept as byte scans** — the attribute-separator check and the PI-target split. Both use `is_ascii_whitespace`, never `char::is_whitespace`, which is Unicode-aware and would split on characters XML does not treat as whitespace.
- **Moved to `str`** — splitting on `:` in `expanded_name`/`resolve_ns`, where no char boundary can fall inside a single-byte codepoint, and the whitespace-outside-root test, written as the four XML whitespace characters explicitly.
- **`OpenElement` moved `Vec<u8>` → `String`** — the last byte holdout. Its helpers had already moved, so every comparison was crossing a boundary for nothing.
- **Five UTF-8 branches deleted, not retyped** — the reader guarantees UTF-8 now, and three of the five silently skipped or discarded input on failure.

### Verification

A parser that compiles, passes, and then mis-parses is the failure mode this change is exposed to, so the unit suite alone was not the argument.

- 654 tests, clippy and fmt clean, `cargo build --locked` in `fuzz/`
- `reply_parser_is_total` — 6.06M fuzz executions on the migrated parser, no crashes
- `fragment_embeds_cleanly` — 8.1M on the pre-migration parser, 5.5M on the migrated one
- **Live vSRX (Junos 24.4R1.9), against a baseline**: 23 + 9 tests passed on both, under `RUSTNETCONF_TEST_VSRX_REQUIRED=1` so a run that never reached the device would have failed rather than reported a pass it did not earn

## What's New in v0.16.1

Patch release, no API change. Two escapes from the outbound XML validator and one entity-resolution bug in the CLI (#96).

`rustnetconf-yang` does **not** bump. It has no source change, and its `rustnetconf = "0.16"` requirement already resolves to 0.16.1 — publishing it would ship a version whose only content is a version number. The 0.15 and 0.16 releases moved all three crates because the requirement itself changed incompatibly; a patch does not do that.

- **The `netconf diff` entity bug** — this is the one that reached users. `rustnetconf-cli`'s copy of `resolve_entity_ref` still called quick-xml's `resolve_predefined_entity`. Under Cargo feature unification, any crate in the graph enabling quick-xml's `escape-html` turns that into a resolver for the full HTML5 entity set, so `netconf diff` could resolve entities the device never sent. The library crate's copy was fixed for exactly this; the duplicate in the CLI was missed. Anyone on `rustnetconf-cli` 0.3.7 should upgrade.

  The CLI's requirement on the library is raised from `0.16` to `0.16.1` at the same time. `rustnetconf-cli` calls `validate_xml_fragment` directly, and `^0.16` permits 0.16.0 — which is what a `cargo install --locked` would have selected from the packaged lockfile, shipping the new CLI against the pre-fix validator.

- **Reserved `xmlns` prefix on an element** (rule 16). `<xmlns:a/>` is two valid NCNames, so the QName check accepted it and no conforming parser does (Namespaces in XML §3). It cannot live in `validate_name` because the rule is asymmetric — `xmlns:p="…"` on an *attribute* is the declaration syntax. Only the prefix is reserved; `<xmlns/>` stays legal.

- **References in attribute values** (rule 17). XML 1.0 §2.3 gives `AttValue ::= '"' ([^<&"] | Reference)* '"'`, so a bare `&` is as illegal as a bare `<`, which was already checked. quick-xml keeps attribute values raw and emits no `GeneralRef` event, so `<i n='&'/>`, `&cmp;` and `&#4;` all passed. Resolution now routes through the same helper as the text path, so attribute values and character data cannot disagree about which entities exist.

Both validator rules were already written down in `fuzz/README.md` as known escapes of the rule list, tracked under #89. One class remains — duplicate *expanded* attribute names — and the README's warning that the rule list is not a conformance check still stands. For a conforming parse, use the opt-in `strict-validation` feature.

### Live-device tests fail loudly now

`tests/integration_vsrx.rs` early-returns when `RUSTNETCONF_TEST_VSRX_HOST` is unset, and libtest has no "skipped" outcome for a test that returns normally — so a run that never contacted a device printed the same `23 passed` as one that did. Set `RUSTNETCONF_TEST_VSRX_REQUIRED=1` when a run is meant to be device coverage: every skip becomes a failure that names its reason, so a misspelled variable or an unreadable key file fails the run instead of passing it. A `~/` in `RUSTNETCONF_TEST_VSRX_KEY` is also expanded now — Rust does not do it, and neither does every shell in that position, and the resulting failure looked like an authentication fault.

## What's New in v0.16.0

Feature release. Nothing was removed from the public API and no existing item changed shape. `rustnetconf-yang` takes a minor bump to 0.2.0 because #81 gave it a real feature flag; `rustnetconf-cli` (0.3.7) has no source changes and bumps only to carry the new `rustnetconf = "0.16"` requirement.

**One source-level break.** `ProtocolError` gains an `InvalidValue` variant, and the enum is not `#[non_exhaustive]`, so downstream code matching it exhaustively needs a new arm or a wildcard:

```rust
match err {
    ProtocolError::CapabilityMissing { .. } => ...,
    // ...
    _ => ...,  // add this, or an explicit InvalidValue arm
}
```

Nothing else in the crate is affected — every other new enum (`WithDefaults`, `DeleteTarget`, `CopySource`, `ConfigLocation`) is new, not an addition to an existing one. No public error enum is `#[non_exhaustive]` today, so variant additions will keep landing as breaks until that changes.

### Protocol coverage

Four gaps closed, each one an operation the module docs already implied existed.

- **`copy-config`, `delete-config`, `cancel-commit`** (#71). RFC 6241 §7.3, §7.4 and §8.4. `delete_config` takes a `DeleteTarget`, not a `Datastore`, because the `ietf-netconf` YANG module's `config-target` choice contains only `startup` and `url` — neither `running` nor `candidate` is a legal target, and a type that cannot express them is better than a runtime rejection.

- **XPath filters** (#72). `XPathFilter` with namespace bindings, for `get` and `get-config`. The module documentation had promised this for several releases; it now exists.

- **`with-defaults`** (#74), RFC 6243. `get_config_with_defaults`, `get_with_defaults`, `copy_config_with_defaults`, covering all four retrieval modes.

- **Partial lock** (#73), RFC 5717. `partial_lock` / `partial_unlock`. If a partial-lock reply is lost, the session is marked unhealthy rather than assumed unlocked — a lock whose state is unknown is not a lock you may act on, and a pooled connection must not carry that ambiguity to the next caller.

### Reading large replies

- **Streaming reads** (#76). `get_config_streaming`, `get_streaming` and `stream_rpc` write the raw reply envelope to an `AsyncWrite` as frames arrive, so peak memory is one frame rather than the whole document. A cancelled future or a failing sink poisons the session instead of leaving a partial reply in the buffer for the next RPC — or for whoever the pool hands the connection to next.

- **The read-buffer ceiling is adjustable** (#92). `set_max_read_buffer` on `Client` and `Session`, and on `TlsClientBuilder`. It survives `reconnect()` and is restored when a pooled connection is returned, so a raised ceiling is a property of the client rather than of one connection that happens to still be open.

### XML correctness

- **Fuzzing found fourteen escapes from the outbound validator** (#80). `validate_xml_fragment` is a hand-written conformance layer over quick-xml, which is deliberately permissive. Differential fuzzing against a conforming parser found fourteen fragments it accepted that are not well-formed XML — including one where the validation mechanism was itself the escape: `</_>x<_>` balanced against the synthetic root the validator wraps the fragment in. All fourteen are fixed, and the fuzz targets ship in `fuzz/`.

- **Strict validation is available, opt-in** (#89, #93). The `strict-validation` feature validates by attempting a conforming parse instead of by enumerating rules. Off by default: it costs one small pure-Rust dependency (142 → 143 crates), and that is the caller's trade to make. Three residual cases the rule list still accepts are documented on #89.

### Supply chain and build

- **The stale `RUSTSEC-2023-0071` suppression is gone** (#78). `rsa` had already left the dependency graph — russh 0.62 gates it behind a non-default feature — so the suppression was a loaded gun: it would have silently covered a genuine future `rsa` advisory.

- **`chacha20` moved off a yanked version.** The lockfile pinned 0.10.0, which the registry has since yanked; it now resolves to 0.10.2. Transitive via russh (`rand`, `ssh-cipher`), and not a published advisory — `cargo audit` reports it as a warning, not a vulnerability — but a release should not ship a lockfile pinning a yanked crypto crate.

- **`libyang2` bundling is now opt-out** (#81). `rustnetconf-yang` built libyang2 from source unconditionally, costing ~44 MB of build artifacts and making `cmake` a hard prerequisite. `default = ["bundled"]` keeps that behaviour; `--no-default-features` links a system libyang2 via pkg-config instead.

- **YANG generated code is now committed; libyang2 becomes maintainer-only** (#105). The build script that ran libyang2 on every build is deleted. Generated Rust types live committed at `rustnetconf-yang/src/generated.rs` and ship with the published crate. Consumers link no libyang2 and need no cmake. The code generator moved to a bin target gated on a new `regenerate` feature — regenerate with `cargo run -p rustnetconf-yang --features regenerate --bin codegen`. CI runs it and fails on drift, so a stale committed file cannot merge. The `bundled` feature is now inert unless `regenerate` is also enabled; system-libyang is `--no-default-features --features regenerate`.

- **`unsafe_code = "forbid"` is enforced workspace-wide** (#79). The README claimed no unsafe code; a crate-root `#![forbid(unsafe_code)]` does not reach build scripts, examples or integration tests, which are separate crates. It is now a `[workspace.lints]` entry, so the claim is enforced everywhere it was being made.

## What's New in v0.15.0

Breaking change (#66). One variant changes shape; nothing else in the public API moves. `rustnetconf-cli` (0.3.6) and `rustnetconf-yang` (0.1.6) have no source changes and bump only to carry the new `rustnetconf = "0.15"` requirement, exactly as they did at 0.14.0.

- **`RpcError::ServerError` now carries a boxed struct.** Its 7 RFC 6241 §4.3 fields moved into a new public `RpcServerError`, and the variant became `ServerError(Box<RpcServerError>)`. The fields are unchanged, still public, and still all present — only where they live moved.

  ```rust
  // before
  Err(NetconfError::Rpc(RpcError::ServerError { tag, message, .. })) => ...

  // after
  Err(NetconfError::Rpc(RpcError::ServerError(e))) => ... // e.tag, e.message
  ```

  Construction gets `From<RpcServerError>` for both `RpcError` and `NetconfError`, so `rpc_server_error.into()` boxes for you. `Display` output is byte-for-byte what it was — code matching on the error *string* is unaffected.

- **Why: the error types were exactly 128 bytes.** `NetconfError` and `RpcError` both measured 128, which is precisely clippy's `result_large_err` threshold, so every fallible function in the public surface tripped the lint — 71 sites once the suppression was lifted. `ServerError` was the whole of it: `message` plus three `Option<String>` is 96 bytes before `ErrorTag`'s own 24. Boxing that one variant takes `RpcError` 128 → 48 and `NetconfError` 128 → 72, and the ceiling becomes `TransportError::HostKeyMismatch` at 72.

  Every `Result<T, NetconfError>` in the crate was moving 128 bytes on the success path too, since a `Result` is as wide as its larger arm.

- **The suppression is gone, not relocated.** `#![allow(clippy::result_large_err)]` in `src/lib.rs` and two local allows in `src/session.rs` were removed; `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes clean without them. A new test asserts each error enum stays under 128 bytes, so a future inline `String` field fails the suite rather than quietly re-arming the lint.

- **The toolchain is now pinned.** `rust-toolchain.toml` pins 1.98.0, matching every sibling crate. CI installed `stable` unpinned, which is why a clippy release turned the gate red with no change on this side.

## What's New in v0.14.5

Bug-fix release (#67). `rustnetconf-cli` and `rustnetconf-yang` are unchanged. No API changes — a drop-in patch upgrade from 0.14.4.

- **Fixed: a benign Junos warning no longer sinks the whole load.** Deleting a statement that is not present on an SRX345 returns an `<rpc-error>` carrying only severity and message — RFC 6241 makes `error-type` and `error-tag` mandatory, and Junos omits both. `RpcErrorBuilder::finish` rejected the reply with "rpc-error is missing error-type", so a warning failed the load and took `commit_check_config` and `apply_junos_change_set` with it — the same tools 0.14.4 had just unblocked on the same device.

- **The tolerance is keyed on severity.** A missing `error-type`/`error-tag` is accepted only when severity is `warning`: a warning carries no verdict, so an absent classification costs nothing, while an `error`-severity `rpc-error` still has to say what it is rather than be reported under an invented tag. `RpcErrorInfo` already modelled `error_type` as `Option`, so the struct did not change; a missing tag becomes `ErrorTag::Other("unspecified")`.

  Severity is read but not consumed before the type and tag checks, so the strict path still reports error-type, then error-tag, then error-severity in that order — an empty `<rpc-error/>` fails exactly as it always did.

## What's New in v0.14.4

Bug-fix release (#65). `rustnetconf-cli` and `rustnetconf-yang` are unchanged. No API changes — a drop-in patch upgrade from 0.14.3.

- **Fixed: a standalone SRX's commit-check verdict was lost.** A single-RE SRX345 answers a commit-check with a closed `<commit-results>` followed by a sibling `<ok/>`. RFC 6241 does not allow a payload and `<ok/>` together, so the reply failed to parse with "`<ok/>` conflicts with an existing payload" and the verdict was discarded even though the check had passed. This made the governed write path unusable on the lab's physical SRX345 while the ungoverned one worked ([rustjunosmcp#358](https://github.com/mechubsec/rustjunosmcp/issues/358)).

  The chassis-cluster form of the same reply already parsed, because there Junos leaves `<routing-engine>` unclosed — the payload is still open when `<ok/>` arrives. That existing tolerance was keyed on depth > 0, so it covered only the malformed shape: a device that closed the element correctly fared *worse* than one that did not.

- **The scope is deliberately narrow.** Exactly one `<ok/>` after a closed `<commit-results>` is tolerated. An earlier draft accepted `<ok/>` after any closed direct payload, which turned `<software-information>…</software-information><ok/>` into `RpcReply::Ok` — handing the caller an empty string and silently dropping a body it had asked for. Trading a loud parse error for silent data loss is the wrong trade, so every other direct payload keeps the conflict, as do a duplicate `<ok/>` and any direct sibling following the tolerated one. A hard `<rpc-error>` still wins, so a failed check is never reported as passing. All four boundaries are pinned by tests, against a fixture captured off the wire from the SRX345 on Junos 21.2R3-S6.11.

- **Also: `clippy::result_large_err` suppressed.** Stable 1.98.0 turned it into a red gate on main. Suppression was the right scope for a patch release carrying a production bugfix; the lint is fixed properly in 0.15.0 (#66).

## What's New in v0.14.3

Bug-fix release (#61). `rustnetconf-cli` and `rustnetconf-yang` are unchanged. No API changes — every function touched is private — so a drop-in patch upgrade from 0.14.2.

- **Fixed: a cancelled commit no longer misclassifies the next unrelated failure.** `Session` tracked "a commit is in flight" in a `pending_commit` field, set before the send and cleared after it. Drop that future at its `.await` — a `tokio::time::timeout`, a `select!` — and the reset never ran, so the flag stayed set for the life of the session. The next unrelated transport EOF, during a `get-config` or anything else, was then reported as `RpcError::CommitUnknown`.

  That is the flag's purpose inverted. It exists so a genuinely indeterminate commit is not mistaken for a clean I/O failure; the leak made a clean I/O failure look like an indeterminate commit, which can send a caller hunting for a commit that never happened or stop it retrying something perfectly safe to retry.

- **The fix removes state rather than guarding it.** "This is a commit" describes one call, not the session, so it is now an `is_commit` parameter threaded through the private `send_rpc_raw` → `read_rpc_reply` → `read_message` chain, and the field is gone. Cancellation-safety follows by construction: the fact lives in the call frame and dies with any future that is dropped. Commit paths call a new private `send_rpc_commit()`; the other 20 `send_rpc` call sites are untouched.

  An RAII guard was considered and rejected — holding `&mut self.pending_commit` across `self.send_rpc(&mut self)` does not borrow-check, so it would have forced the flag into `Rc<Cell<bool>>`/`Arc<AtomicBool>`, adding an allocation and indirection to every session to fix an error path.

- **Regression test.** `test_cancelled_commit_does_not_poison_later_eof` cancels a commit mid-await against a transport that parks, then asserts a following non-commit EOF is reported as a transport error. It was verified to **fail** against the 0.14.2 implementation, returning `CommitUnknown`, so it guards the fix rather than passing vacuously. A `StallingMockTransport` was added for it, because the existing mock reads synchronously and can never be interrupted mid-await.

## What's New in v0.14.2

Follow-up to 0.14.1, from wiring the consumer side in [rustez#41](https://github.com/mechubsec/rustez/issues/41) (#60). `rustnetconf-cli` and `rustnetconf-yang` are unchanged — their `rustnetconf = "0.14"` requirement already matches.

No API removals and no source-breaking changes, so a drop-in patch upgrade. One deliberate behaviour change, on an error path only: a Junos `commit-configuration` that disconnects before its reply now reports `CommitUnknown` instead of a generic transport error — see the second bullet.

- **New `Session::commit_configuration_with_log()` / `Client::commit_configuration_with_log()`** — additive. A Junos commit carrying a log comment, visible in `show system commit`. The log text is XML-escaped for you. Clears the candidate-dirty flag on success, exactly as `commit_configuration()` does.

  `commit_configuration()` takes no arguments and `commit_configuration_xml()` hard-coded a bare `<commit-configuration/>`, so a caller wanting Junos's `<log>` child had to build the fragment and send it through raw `rpc()`. Every method that clears the dirty flag does so as a private side effect, and there is no public way to clear it — so that raw path could not. The session stayed marked dirty across a commit that genuinely cleaned the candidate, and `close_session()` then discarded afterwards. Harmless for that session's own work; on the **shared** Junos candidate it destroys anything another operator staged between the commit and the close. This is the mirror of #58: that was an RPC that could not atomically *set* the flag, this was a commit that could not *clear* it.

- **`commit_configuration()` keeps its signature and its XML.** Both methods now delegate to one private helper; the no-log path emits byte-identical XML, asserted in a test. `rpc::operations::commit_configuration_xml()` also keeps its one-argument form — the log variant is a separate `commit_configuration_with_log_xml()`, because that module is public API and changing the arity would break existing callers at source.

- **Fixed: a Junos commit that disconnects before its reply now reports `RpcError::CommitUnknown`.** `commit()` and `confirmed_commit()` already bracketed their send with the pending-commit state; `commit_configuration()` never did, so a mid-commit disconnect surfaced as a generic transport EOF. The device may have applied the commit, and a caller that reads that as a clean failure can retry a commit that already took effect. Both `commit_configuration()` and the new log variant now signal it correctly. This is a behaviour change on an error path only — the success path is untouched.

## What's New in v0.14.1

Additive follow-up to 0.14.0 for `rustnetconf` (0.14.1), from wiring the consumer side in [rustez#36](https://github.com/mechubsec/rustez/issues/36) (#58). `rustnetconf-cli` (0.3.5) and `rustnetconf-yang` (0.1.5) are unchanged — their `rustnetconf = "0.14"` requirement already matches. No behaviour changes and no API removals: a drop-in patch upgrade from 0.14.0.

- **New `Session::rpc_candidate_change_with_warnings()` / `Client::rpc_candidate_change_with_warnings()`** — additive. The warnings-returning counterpart of `rpc_candidate_change()`, returning the same `(String, Vec<RpcErrorInfo>)` tuple as `rpc_with_warnings()` with the same preflight-then-mark ordering.

  0.14.0 left the warnings path without an atomic option. A caller who needed the warnings back from a candidate-modifying RPC had only `rpc_with_warnings()` plus a hand call to `mark_candidate_dirty()`, and that is not equivalent: `rpc_with_warnings()` validates the fragment and returns *before sending anything*, so a hand-marked malformed fragment left the candidate marked dirty for an RPC that never reached the device — and the next `close_session()` would then send `<discard-changes/>` against another operator's uncommitted work. That is the #55 bug reached by a different route. Hand-marking also could not replicate the keepalive and session-state preflight, since neither is public.

- **No change needed if you already use `rpc_candidate_change()`.** Its ordering and behaviour are identical; the sequence simply moved into a shared internal helper that both methods now call.

## What's New in v0.14.0

Junos candidate-datastore safety for `rustnetconf` (0.14.0), from a bug reproduced on hardware (#55, PR #56). `rustnetconf-cli` (0.3.5) and `rustnetconf-yang` (0.1.5) carry the new dependency requirement but have no source changes.

**This is a behaviour change on close, which is why it is a minor bump and not a patch.** Read the last bullet before upgrading if you send candidate-modifying RPCs through the raw `rpc()` escape hatch.

- **`close_session()` no longer discards a candidate this session never touched** (#55). Junos returns `CloseSequence::DiscardThenClose`, and the discard fired unconditionally — including for sessions that only read. On a standalone Junos device the candidate datastore is *shared*, so closing such a session destroyed uncommitted work belonging to an operator at the CLI or to another NETCONF client. The session now tracks whether it dirtied the candidate and discards only then.

  The tracking is deliberately asymmetric. `edit_config` (candidate target only), `load_configuration`, and `rollback_configuration` mark *before* sending, so a partial or failed edit still counts; `commit`, `commit_configuration`, `confirmed_commit`, `discard_changes`, and `close_configuration` clear it only on success. Anything that fails after the write begins still marks dirty, because a partially applied change does need cleaning up.

- **New `Session::rpc_candidate_change()` / `Client::rpc_candidate_change()`** — additive. The supported way to send a vendor-specific candidate-modifying RPC (Junos `<load-configuration>` and friends) through the raw path. It validates the fragment, completes the send preflight, marks the candidate dirty, and only then writes, so a locally rejected fragment or a failed keepalive probe cannot leave a false mark that a later close would act on.

- **New `mark_candidate_dirty()` / `candidate_dirty()`** on both `Session` and `Client` — additive, for callers who need manual control. Prefer `rpc_candidate_change()`: marking by hand before an RPC that never reaches the device reintroduces exactly the bug above.

- **Upgrade note.** If you send candidate-modifying RPCs through raw `rpc()`, you were previously relying on the unconditional discard to clean up after them. That cleanup is now conditional and will not fire for those calls. Switch them to `rpc_candidate_change()`. Everything going through the typed operations (`edit_config`, `load_configuration`, `rollback_configuration`) is tracked automatically and needs no change.

## What's New in v0.13.3

Maintenance release for `rustnetconf` (0.13.3). `rustnetconf-cli` (0.3.4) and `rustnetconf-yang` (0.1.4) are unchanged. No public API changes — a drop-in patch upgrade from 0.13.2.

- **RPC reply parsing hardened** (#49). Reply parsing and repair moved into a dedicated `reply` module with tightened handling of malformed and partial replies. `parse_rpc_reply`, `RpcReply`, and `RpcErrorInfo` keep their existing paths and signatures via re-export.
- **russh 0.62.4 → 0.62.5** (#50). Picks up the `Channel::data()` backpressure fix. The GHSA-m65r-rprj-r5rg advisory in that release is server-side only; this crate uses `russh::client` exclusively and was never exposed to it.
- **cmov 0.5.3 → 0.5.4** (#46).
- **License gating in CI.** A `deny.toml` allow-list now exists, and CI runs `cargo deny check bans sources licenses`. Previously licenses were ungated, because with no config cargo-deny rejects every license by default.
- **Scope documented** — see [Scope](#scope). A native SCP1 client was added (#52) and reverted (#53) within this cycle, before any release; issues #47 and #51 were closed on the same grounds. No released version ever contained it.

## What's New in v0.13.0

Vendor-profile plumbing for `rustnetconf` (0.13.0) and `rustnetconf-cli` (0.3.4), from a project review (PRs #37, #38). `rustnetconf-yang` is unchanged at 0.1.4.

- **`DevicePool` honors an explicit vendor profile.** `DeviceConfig.vendor` is now wired into connection setup — previously it was silently ignored. The field changed from `Box<dyn VendorProfile>` to `Arc<dyn VendorProfile>`; new `ClientBuilder::vendor_profile_arc()` and `Session::set_vendor_profile_arc()` support the shared-ownership path. The existing `vendor_profile(Box<…>)` API is unchanged.
- **New `Client::unwrap_config()`** (and `Session::unwrap_config()`) exposes the connected device's vendor `unwrap_config` — additive public API.
- **Vendor-aware CLI diffs.** `netconf plan`/`apply` now normalize the desired config through the connected device's vendor profile instead of a hardcoded `<configuration>` strip. This fixes an asymmetric, potentially wrong diff against generic (non-Junos) devices; Junos behavior is unchanged.
- Internally, `VendorProfile::post_facts_hook` takes `&self` (with `JunosVendor` cluster state moved to interior mutability) so profiles work under `Arc`.

## What's New in v0.12.3

Parser correctness patch for `rustnetconf` (0.12.3) and `rustnetconf-cli` (0.3.3), from a full code review (closes #33 via PR #34). No API changes; `rustnetconf-yang` is unchanged at 0.1.4.

- **Namespace prefixes preserved** when reconstructing `<data>`/`error-info`/Junos inner XML — `<if:interfaces xmlns:if="…">` no longer collapses to `<interfaces>`.
- **CDATA sections captured** in every parser (reply data, error fields, capabilities, eventTime, session-id, CLI diff) instead of being silently dropped.
- **Attribute values fully re-escaped** on reconstruction, so single-quoted source attributes containing `"` can't produce malformed output.
- **Text-format Junos config is XML-escaped** in `load_configuration` — set commands containing `&`/`<`/`>` previously generated malformed RPCs.
- **Top-level empty elements** under `<rpc-reply>` (e.g. `<software-information/>`) now parse as data instead of a bare `<ok/>`.
- Dependency hygiene: `anyhow` 1.0.103 (clears RUSTSEC-2026-0190) and un-yanked `crypto-bigint` 0.7.5 — `cargo audit` is fully clean.

## What's New in v0.12.2

Security patch for all three crates (`rustnetconf` 0.12.2, `rustnetconf-cli` 0.3.2, `rustnetconf-yang` 0.1.4), closing #31. No API changes.

- **quick-xml 0.37 → 0.41:** clears two RUSTSEC advisories in the XML parser that handles device-returned NETCONF data — [RUSTSEC-2026-0194](https://rustsec.org/advisories/RUSTSEC-2026-0194) (quadratic duplicate-attribute check) and [RUSTSEC-2026-0195](https://rustsec.org/advisories/RUSTSEC-2026-0195) (unbounded namespace-declaration allocation, memory-exhaustion DoS).
- **Entity handling adapted to quick-xml 0.38+ semantics:** entity references (`&amp;`, `&#38;`, …) now stream as separate `GeneralRef` events instead of arriving decoded inside text. All reader loops accumulate and resolve them, so element values containing `&`, `<`, `>` (Junos descriptions, URLs, error messages) round-trip whole instead of being silently truncated. Covered by new round-trip regression tests in every parser.
- **Dropped the unused `serialize` (serde) feature** of quick-xml in `rustnetconf-yang` — the crate only uses the manual `Writer` API.

## What's New in v0.12.1

Security and robustness patch for `rustnetconf` (0.12.1) and `rustnetconf-cli` (0.3.1). No API changes.

- **Well-formed reconstructed XML:** the RPC reply parser now re-escapes decoded entities (`&`, `<`, `>`) when reconstructing `<data>`, `error-info`, and Junos inner content. Previously `unescape()`-decoded text was re-emitted raw, which could yield malformed XML for device data containing special characters. Covered by new regression tests.
- **Cleared yanked dependency:** bumped `russh` 0.60 → 0.61 and `aes` to 0.9.1, clearing the `cargo audit` yanked-crate warning (and reducing the dependency count).
- **Non-Unix state-file safety:** on non-Unix platforms (no portable `chmod`), `netconf` now emits a warning that saved state snapshots are not guaranteed owner-only and documents the caveat, since snapshots may contain sensitive device config.

## What's New in rustnetconf-yang v0.1.3

Documentation-only patch release of the `rustnetconf-yang` crate (closes #30). No API or behavior changes.

- Module docs now state that generated types require enabling the `generated` Cargo feature (off by default) — without it the `ietf_*` modules are absent.
- Clarified that codegen reads the crate's own bundled `yang-models/`, not the consumer's project; documented how to vendor the crate to use custom YANG files.
- Corrected the usage example to the real generated API (`Option`-wrapped fields, `type` leaf emitted as `type_`).

## What's New in v0.12.0

OpenSSH `known_hosts`-style host-key pinning for fleet operation. Merged via PR #29 (closes #28).

**New features:**
- `HostKeyVerification::KnownHosts(PathBuf)` — verify the server's SHA-256 fingerprint against an OpenSSH `known_hosts(5)` file on every connect. Supports plain hostnames, `[host]:port`, wildcards (`*`/`?`), CIDR networks, hashed `|1|salt|hmac-sha1` entries, and `@revoked` markers. The file is re-read on every connect — no caching, so external rotation tools are picked up immediately.
- New structured errors on `TransportError`: `HostKeyMismatch { host, expected, actual }`, `HostKeyNotInKnownHosts { host, port, path }`, `HostKeyRevoked { host }`.
- CLI: new optional `known_hosts_path` field on `[devices.*]` and `[defaults]` in `inventory.toml`. Per-device value wins; setting both `host_key_fingerprint` and `known_hosts_path` on the same device is a hard error.
- `examples/known_hosts.rs` demonstrates the `ssh-keyscan` → `KnownHosts(path)` workflow with comments on each failure mode.

**Breaking changes:**
- `DeviceConfig` (the connection-pool config struct) gained a new field `host_key_verification: Option<HostKeyVerification>`. Existing struct-literal callers must add the field. `None` means "use library default" (`RejectAll` since v0.11.0).

**Quality:**
- Live-device integration tests (`integration_vsrx`, `integration_vendor_pool`) are now opt-in via `RUSTNETCONF_TEST_VSRX_HOST` — without it, the suite is a clean no-op for contributors without a Junos lab.

## What's New in v0.11.0

Security remediation pass — addresses the seven findings from the internal
audit (RNC-SEC-001..006 + CI hardening). Merged via PR #27.

**Breaking changes:**
- `ClientBuilder` default `HostKeyVerification` is now `RejectAll` (fail closed). Connections refuse to complete until the caller pins a fingerprint or explicitly opts in to `AcceptAll`. `ProxyJump` hops parsed from `~/.ssh/config` likewise default to `RejectAll`.
- `inventory.toml`: device `password` and `key_passphrase` fields now deserialize into a `SecretString` newtype. `Debug` prints `SecretString(***)` and contents zeroize on drop.
- `ClientBuilder::password` / `.key_passphrase` now accept `Option<Zeroizing<String>>` (was plain `Option<String>`).

**Security fixes:**
- **RNC-SEC-001** — SSH host-key verification fails closed by default in both the library and the CLI. New CLI flag `--insecure-accept-host-key` for lab use; otherwise `host_key_fingerprint` must be set per device in `inventory.toml`.
- **RNC-SEC-002** — RUSTSEC-2023-0071 (rsa Marvin Attack timing side-channel) risk-accepted via `.cargo/audit.toml` with reachability analysis and review date 2026-08-01. russh bumped 0.60.2 → 0.60.3.
- **RNC-SEC-003** — Inventory passwords use the new `SecretString` type with redacted `Debug` and a custom `Deserialize` that zeroizes on drop.
- **RNC-SEC-004** — State files always land at `0o600` via atomic temp-file + `rename(2)`, even when a pre-existing file had looser permissions. `.netconf` and `.netconf/state` are forced to `0o700` on every call. Stale temp files from a prior crash are cleaned up.
- **RNC-SEC-005** — `apply` and `rollback` guarantee candidate-lock cleanup on error. New `Client::release_candidate_lock_best_effort` (discard-changes + unlock, swallowing errors) is invoked from extracted `*_locked_region` helpers.
- **RNC-SEC-006** — Desired XML is validated for well-formedness *before* any device connection or candidate lock. Errors name the offending file.

**CI hardening:**
- New `.github/workflows/ci.yml` runs build, test (workspace, all features), clippy with `-D warnings`, rustfmt, and `cargo audit` on every push and PR to main.

**Quality fixes:**
- YANG codegen: generated `use super::*;` / `use crate::serialize::*;` imports now carry `#[allow(unused_imports)]` so modules without leaf references compile cleanly under `-D warnings`.
- YANG codegen test: corrected `r#type` → `type_` to match the field-sanitization the generator actually emits.

## What's New in v0.10.0

**Breaking changes:**
- `HostKeyVerification` no longer implements `Default` — callers must explicitly choose a host key policy
- `SshAuth::Password` and `SshAuth::KeyFile { passphrase }` now use `Zeroizing<String>` instead of `String`
- User-provided XML content (RPC bodies, filters, configs) is now validated for well-formedness before sending

**Security fixes:**
- Shell injection via ProxyCommand `%h`/`%p` substitution — values are now shell-escaped
- Credentials (passwords, passphrases) zeroized on drop via the `zeroize` crate
- XML fragment validation prevents injection through malformed RPC content
- TLS `danger_accept_invalid_certs` now emits a detailed warning about the full scope of the bypass
- CLI device names validated to prevent path traversal; state files written with `0600` permissions

**New features:**
- Configurable RPC timeout (`.rpc_timeout(Duration)`) — prevents indefinite blocking on unresponsive devices
- Configurable read buffer size (`.max_read_buffer(bytes)`) — defaults to 100 MB
- IPv6 address support — bracket notation (`[::1]:830`) and bare IPv6 addresses
- Capability normalization — legacy Junos capability URIs are mapped to standard URIs during session establishment

**Quality improvements:**
- Connection pool health checks — dead connections are discarded on checkout and drop instead of being recycled
- Blocking `std::fs::read_to_string` in async context replaced with `tokio::fs`
- Unnecessary `Arc<Mutex<>>` removed from `SshTransport`
- `AtomicU64` message counter replaced with plain `u64` (Session is `&mut self` only)
- YANG codegen: full container/list XML serialization, complete Rust keyword list, hard error on module load failure
- CLI: plan summary fixed for non-JSON mode, diff engine compares all list elements
- Removed unused `futures` dependency and `quick-xml` serialize feature
- `ErrorTag` implements `std::str::FromStr`; `Session::validate()` checks `:validate` capability
- Dependency updates: russh 0.60.2, rustls 0.23.40, tokio 1.52.2, rustls-webpki 0.103.13

