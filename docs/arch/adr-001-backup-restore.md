# ADR 001: Enclave Backup and Restore

## Changelog

* 2025‑09‑01: Initial draft.

## Status

Accepted

## Context

Quartz enclaves derive secret material (e.g., enclave key) that must survive routine restarts and, ideally, catastrophic
host failures. We consider two failure modes:

1. **Planned reboot / host maintenance** — the enclave process restarts on the _same_ machine and needs to resume
   operation without renegotiating secrets.
2. **Primary enclave failure** — the original machine dies; a _different_ TEE instance must take over with the same
   secrets.

We propose two complementary protocols implemented in phases:

- **Protocol-1 (this ADR):** Local on-disk backup/restore using *sealed files* to cover reboots.
- **Protocol-2 (future work):** Replay-protected key-exchange to another TEE using Quartz’s on-chain messaging and
  light-client verification to cover host failures.

The immediate goal is to de-risk reboots with a minimal, auditable surface: sealed files plus a narrowly-scoped restore
path that bypasses handshake safely and predictably.

## Decision

### Protocol-1: On-disk sealed backup/restore (implement now)

**High-level design**

- After the initial handshake completes, the enclave *exports* all state necessary to resume a valid post-handshake
  state and writes it as a **sealed file** at a well-known path. Under Gramine, this uses Protected Files so ciphertext,
  integrity, and rollback protection are enforced at the file layer.
- On (re)start, if a sealed backup exists, the enclave *imports* that state, signals “handshake complete” to the host,
  and resumes operation **without** re-running the handshake. If no backup is present, the standard handshake runs and a
  new backup is written afterward.

**Interfaces (framework-level, generic, hard to misuse)**

We define minimal traits to make backup composition explicit and bijective:

```rust
/// Rudimentary backup and restore functionality
#[async_trait::async_trait]
pub trait Backup {
    type Config;
    type Error: Send + Sync;

    async fn backup(&self, config: Self::Config) -> Result<(), Self::Error>;
    async fn has_backup(&self, config: Self::Config) -> bool;
    async fn try_restore(&mut self, config: Self::Config) -> Result<(), Self::Error>;
}

/// Export/Import are bijective views of enclave sub-systems.
/// They're very much alike serde except they're async.
#[async_trait::async_trait]
pub trait Export {
    type Error: Send + Sync + Debug;

    async fn export(&self) -> Result<Vec<u8>, Self::Error>;
}

#[async_trait::async_trait]
pub trait Import: Sized {
    type Error: Send + Sync + Debug;

    async fn import(data: Vec<u8>) -> Result<Self, Self::Error>;
}
```

Enclave subsystems (`Store`, `KeyManager`, `Attestor`, `Ctx`) implement Export/Import, making the backup a pure
composition of their byte images. The default implementation for our concrete enclave bundles these parts:

```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct DefaultBackup {
    store: Vec<u8>,
    key_manager: Vec<u8>,
    attestor: Vec<u8>,
    ctx: Vec<u8>
}

#[async_trait::async_trait]
impl<C, A, K, S> Backup for DefaultEnclave<C, A, K, S>
where
    C: Send + Sync + Export + Import,
    A: Attestor + Clone + Export + Import,
    K: KeyManager + Clone + Export + Import,
    S: Store<Contract=AccountId> + Clone + Export + Import,
{
    type Config = PathBuf; // backup path (sealed file)
    type Error = anyhow::Error;

    async fn backup(&self, path: Self::Config) -> Result<(), Self::Error> {
        // export parts, serialize, write sealed file (via Gramine PF-backed fs)
        Ok(())
    }

    async fn has_backup(&self, path: Self::Config) -> bool {
        path.is_file()
    }

    async fn try_restore(&mut self, path: Self::Config) -> Result<(), Self::Error> {
        // read sealed file, import parts, swap into self, notify host handshake complete
        Ok(())
    }
}
```

#### Host integration (restore-first boot path)

At process start, the host prefers restoration over re-handshake:

```rust
impl Host /* for DefaultHost */ {
    fn serve_with_query(&self, /* ... */) {
        // try to restore from last backup
        if self.enclave.has_backup(self.backup_path.clone()).await {
            info!("found backup; attempting to restore after 30s...");
            busy_wait_iters(3_000_000_000); // see Replay Protection below

            if let Err(e) = self.enclave.try_restore(self.backup_path.clone()).await {
                error!("failed to restore from backup: {e}");
            }
        } else {
            info!("no backup found; waiting for handshake completion...");
        }
    }
}
```

#### Replay protection (host grinding mitigation)

A malicious host can repeatedly restart the enclave to “grind” the backup for secret data.

SGX lacks a trustworthy wall clock/sleep primitive; a deterministic spin in enclave space slows automated grinding
without trusting host timers. This is not a cryptographic protection but reduces feasible attempt rates.

```rust
#[inline(never)]
fn busy_wait_iters(mut iters: u64) {
    use core::sync::atomic::{AtomicU64, Ordering};

    static SPIN_TICK: AtomicU64 = AtomicU64::new(0);

    while iters != 0 {
        // Prevent the loop from being optimized away and provide a tiny side effect.
        std::hint::black_box(SPIN_TICK.fetch_add(1, Ordering::Relaxed));
        // Hint to CPU that we're in a spin loop (x86: emits PAUSE).
        core::hint::spin_loop();

        iters -= 1;
    }
}
```

### Protocol-2: Key-exchange to another TEE (outline, future work)

Goal: Survive machine loss by transferring the primary enclave’s sealed state to a backup enclave on another host,
mediated by Quartz’s replay-protected on-chain messaging.

Flow ->

- Admin brings up a backup enclave and has it produce RA; submits the backup enclave’s pubkey and RA to a contract.
- Contract verifies RA and emits an event.
- Primary enclave observes the event (via on-chain inbox), seals & encrypts its backup to the backup enclave’s pubkey,
  and posts the ciphertext on-chain.
- On primary failure, admin boots the backup enclave with its local sealed bootstrap and instructs the contract to
  deliver the ciphertext; the backup enclave imports and becomes the new primary.

Note: Full implementation details, message schemas, and failure policies are TBD in a dedicated ADR once Protocol-1 is
shipped.

## Consequences

### Positive

- Deterministic recovery from reboots; zero on-chain coordination required.
- Small/auditable surface; leverages Gramine Protected Files for confidentiality/integrity.
- Generic and composable design via Export/Import makes backups hard to misuse and easy to extend.
- Does not modify the handshake protocol nor application code paths.

### Negative

- Care needed for crash-safety and versioning; improper deployments (wrong mount/permissions) can nullify benefits
  because sealed files are only readable by the same h/w enclave instance that wrote it.
- Not sufficient for cross-host failover; Protocol-2 is required for that and introduces on-chain complexity.

### Neutral

- Multi-solver scenarios require policy on when to refresh backups (post-handshake only vs periodic); left to
  operator policy for now.
- Storage overhead is minimal but non-zero; rotation policy determines disk churn.

## References

- [Gramine docs on date and time](https://gramine.readthedocs.io/en/v1.5/devel/features.html#date-and-time)
- [About Time: On the Challenges of Temporal Guarantees in Untrusted Environments](https://vanbulck.net/files/systex23-time.pdf)
