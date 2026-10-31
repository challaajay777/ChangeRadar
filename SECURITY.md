# ChangeRadar Security Policy

ChangeRadar is engineered with a **local-first, security-first, and privacy-first** architecture. Because ChangeRadar inspects and stores file versions across user filesystems, we adhere to rigorous defensive engineering principles to ensure your system and data remain secure, private, and resilient against tampering or corruption.

---

## 1. Threat Model & Security Principles

### 1.1 Zero External Communication
ChangeRadar never transmits file contents, filenames, metadata, hashes, or telemetry over the internet. The only network component is an optional local HTTP REST API that binds strictly to the loopback address (`127.0.0.1`) and requires a pre-shared cryptographic Bearer token.

### 1.2 Principle of Least Privilege
- Neither ChangeRadar Desktop nor the ChangeRadar Daemon requires elevated administrator or `root` permissions.
- ChangeRadar operates entirely within standard user account privileges and accesses only directories explicitly selected by the user.

### 1.3 Memory Safety
- ChangeRadar’s core engine, hashing, CAS storage, and recovery systems are written in 100% safe Rust.
- Memory corruption vulnerabilities (such as buffer overflows, use-after-free, and double-free bugs) are prevented at compile time.

---

## 2. Storage & Filesystem Hardening

### 2.1 Cryptographic Content Verification
- Version objects are stored in a Content-Addressed Store (CAS) indexed by their BLAKE3-256 cryptographic digest.
- Every read from storage verifies the decompressed payload against the expected content hash. If any bit has flipped due to disk degradation or external tampering, the read is rejected immediately with a checksum failure.

### 2.2 Path Traversal Prevention
- All user-supplied and watcher-supplied paths are canonicalized and normalized.
- ChangeRadar rejects relative path traversals (e.g., `../../`) and checks that file operations remain strictly within the monitored directory boundary.
- Symbolic link loops and recursive symlink attacks are detected and bounded.

### 2.3 Safe Recovery Invariants
- **Non-Destructive Restorations**: Before replacing any file on disk, ChangeRadar captures the current disk state into the CAS as a safety backup. Even if a user restores an incorrect version by mistake, the overwritten version can be instantly recovered.
- **Atomic Operations**: All file restorations write first to a temporary file (`.changeradar-restore-tmp-*.tmp`) and then atomically replace the target file using OS-level atomic rename operations (`MoveFileExW` with `MOVEFILE_REPLACE_EXISTING` on Windows, `rename(2)` on Unix). This guarantees that power failure or process termination will never leave a corrupted partial file.

---

## 3. Reporting a Vulnerability

We take the security of ChangeRadar seriously. If you discover a security vulnerability, please report it responsibly:

1. **Do NOT open a public GitHub issue.**
2. Send a detailed report via encrypted email to:  
   **security@changeradar.org** (or to the project maintainers).
3. Include:
   - A description of the vulnerability and potential impact.
   - Exact steps or proof-of-concept to reproduce the issue.
   - Operating system and version of ChangeRadar tested.

### What to Expect:
- We will acknowledge receipt of your vulnerability report within **48 hours**.
- We will provide an assessment and timeline for a patch within **7 business days**.
- Once a fix is verified and released, we will credit you in the security advisory (unless you request anonymity).
