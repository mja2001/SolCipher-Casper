# SolCipher-Casper

Privacy-first secure document sharing dApp on Casper Network.

Port of the original SolCipher (Solana) with full client-side AES-256-GCM encryption, IPFS storage via web3.storage, and Casper Wasm smart contracts handling access control, expiry, and instant revocation.

## Features

- Zero-knowledge: files encrypted in-browser before upload – neither IPFS node nor blockchain ever sees plaintext  
- Permanent or expiring shares (self-destructing links)  
- On-chain access control lists & instant revocation  
- Wallet-derived encryption keys (Casper Signer – no passwords)  
- Single file or batch folder sharing (auto-generates manifest CID)  
- Fully decentralized – no backend, no database, or server  
- Open source & MIT licensed

## Tech Stack

- Smart Contract: Rust → Casper Wasm  
- Frontend: Next.js 14 + Tailwind CSS + casper-js-sdk  
- Wallet: Casper Signer browser extension  
- Storage: web3.storage (IPFS + Filecoin persistence)  
- Encryption: Web Crypto API (AES-256-GCM)

## Project Structure (planned)
SolCipher-Casper/
├── contract/              # Rust Casper smart contract
│   └── casper-cipher/
│       ├── src/
│       └── Cargo.toml
├── app/                   # Next.js frontend (port of original /app)
│   ├── src/
│   ├── public/
│   └── package.json
├── scripts/               # Deployment & testing scripts
├── README.md
└── LICENSE


## Quick Start (once folders are added)

```bash
git clone https://github.com/mja2001/SolCipher-Casper.git
cd SolCipher-Casper

# Frontend
cd app
yarn install
yarn dev

# Contract
cd ../contract/casper-cipher
cargo build --release --target wasm32-unknown-unknown
