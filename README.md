# NEAR Intents TEE Solver Registry

The NEAR Intents TEE Solver Registry is a protocol that enables secure and private execution of NEAR Intents solvers using Trusted Execution Environment (TEE) technology. This project consists of smart contracts for managing solver registration and a server for launching and managing TEE solvers. 

This protocol allows liquidity pools creation for NEAR Intents. Liquidity providers can transfer funds into the pools' smart contracts. Only the solvers who're running within TEE with the approved Docker images can be registered and authorized to operate against the pools' assets.

## Overview

The system consists of two main components:

1. **Smart Contracts**
   - `solver-registry`: Support liquidity pools creation. Manage registration and verification of TEE solvers for each liquidity pool.
   - `intents-vault`: The vault contract that manage the pool's asset within NEAR Intents.

2. **Solver Management Server**
   - A TypeScript-based server that manages the lifecycle of TEE solvers
   - Handles solver deployment and monitoring for each liquidity pool

## Prerequisites

- Rust and Cargo (latest stable version)
- Node.js (v20 or later)
- pnpm package manager
- Docker and Docker Compose
- NEAR CLI
- A NEAR account with sufficient NEAR tokens for funding the ephemeral accounts in each TEE solver

## Project Structure

```
tee-solver/
├── contracts/           # Smart contracts
│   ├── solver-registry/ # Solver registry contract
│   ├── intents-vault/   # NEAR Intents vault contract
│   ├── mock-intents/    # Mock NEAR Intents contract for testing
│   └── mock-ft/         # Mock fungible token for testing
├── server/             # TEE Solver management server
└── scripts/            # Deployment and utility scripts
```

## Setup and Deployment

### 1. Smart Contracts

1. Build the contracts:

Install [`cargo-near`](https://github.com/near/cargo-near) and run:

```bash
make all
```

2. Test the contracts

```bash
make test
```

3. Deploy the contracts:
```bash
cd contracts/solver-registry
cargo near deploy build-reproducible-wasm <contract-id>
```

#### Tools

- [cargo-near](https://github.com/near/cargo-near) - NEAR smart contract development toolkit for Rust
- [near CLI](https://near.cli.rs) - Interact with NEAR blockchain from command line
- [NEAR Rust SDK Documentation](https://docs.near.org/sdk/rust/introduction)


### 2. Solver Launcher Server

Every time a new liquidity pool is created in the Solver Registry contract, the server will find the pools that needs to create a solver. 

We'll use the [TEE-powered AMM solver](https://github.com/think-in-universe/near-intents-tee-amm-solver/tree/feat/tee-solver) as the default solver to launch once a new liquidity pool is created in the solver registry contract.

1. Navigate to the server directory:
```bash
cd server
```

2. Install dependencies:
```bash
pnpm install
```

3. Set up environment variables:
```bash
cp .env.example .env
# Edit .env with your configuration
```

4. Start the server:
```bash
# Development mode
pnpm dev

# Production mode
pnpm build
pnpm start
```

## Security

This project uses TEE (Trusted Execution Environment) to ensure secure and private execution of NEAR Intents solvers. 

## Acknowledgement

The project is inspired by the incredible design of [Shade Agent](https://github.com/NearDeFi/shade-agent-template).

## License

This project is licensed under the MIT License - see the LICENSE file for details.
