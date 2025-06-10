# TEE Solver Management Server

## Prerequisites

- Node.js (v20 or higher)
- pnpm installed: `npm i -g pnpm`

## Installation

1. Clone the repository
2. Install dependencies:

```bash
pnpm install
```

## Building

```bash
pnpm build
```

## Run

```bash
pnpm start
```

## Development

For development, you can use:
```bash
pnpm dev
```

## Environment Variables

Create a `.env` file and add Phala Cloud API key, funding account ID and private key. 

```bash
cp .env.example .env
```

Create a `.env` file in the root directory with the following variables:

```bash
ENV=testnet
PHALA_API_KEY=phak_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
FUNDING_ACCOUNT_ID=xxxxxxxx.testnet
FUNDING_ACCOUNT_PRIVATE_KEY=ed25519:xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```
