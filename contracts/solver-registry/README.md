# Agent Worker Registry Smart Contract

A secure and verifiable smart contract system for managing worker agents in a Trusted Execution Environment (TEE) on NEAR Protocol. This contract provides verification, registration, and access control mechanisms for worker agents.

## Features

- **Worker Registration & Verification**
  - Secure registration of worker agents through TEE attestation
  - Verification of remote attestation quotes and collateral
  - Storage of worker checksums and compose hashes

- **Access Control System**
  - Method-level access control based on verified worker compose hashes
  - Owner-managed approval system for worker compose hashes
  - Protected methods accessible only by verified agents

## Smart Contract Methods

### Core Methods

```rust
// Register a new worker with attestation data
pub fn register_worker(
    quote_hex: String,
    collateral: String, 
    checksum: String,
    tcb_info: String
) -> bool

// Get worker information
pub fn get_worker(account_id: AccountId) -> Worker
```

### Access Control

```rust
// Approve a worker compose hash (owner only)
pub fn approve_compose_hash(compose_hash: String)

// Remove a worker compose hash (owner only)
pub fn remove_compose_hash(compose_hash: String)
```

## How to Build Locally?

Install [`cargo-near`](https://github.com/near/cargo-near) and run:

```bash
cargo near build
```

## How to Test?

```bash
cargo test
```

## Deployment

```bash
cargo near deploy <account-id>
```

## Security Considerations

- All sensitive methods are protected by worker verification
- Worker registration requires valid TEE attestation
- Access control is managed through compose hash verification
- Owner-only administrative functions

## Technical Architecture

1. **Worker Registration Flow**
   - TEE generates attestation quote
   - Contract verifies quote authenticity
   - Worker compose hash and checksum are stored
   
2. **Method Access Control**
   - Methods check caller's registered compose hash
   - Only approved compose hashes can access protected functions
   - Owner manages approved compose hash list

## Useful Links

- [NEAR Rust SDK Documentation](https://docs.near.org/smart-contracts/quickstart)
- [Chain Abstraction Telegram Group](https://t.me/chain_abstraction)
- [Shade Agent Reference](https://near.ai/shade)
