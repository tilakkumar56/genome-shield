# GenomeShield - Private Genomic Matching on Solana

Compare genetic markers without exposing raw sequences. Powered by Arcium MPC. Real end-to-end computation.

Live Demo: https://genome-shield.vercel.app
Program ID: 4kUgT1BdfeMGt2UVPgb1f2iZjvRR8WiSodyYYV2vnM6m (Solana Devnet)

## Real MPC Flow

- RescueCipher.encrypt: Encrypts SNP markers via x25519 ECDH
- queue_computation: Submits to Arcium MPC via Solana program
- awaitComputationFinalization: Waits for ARX nodes callback

## Tech Stack

Solana - Arcium - Arcis - Anchor 0.32.1 - React + Vite - Arcium Client SDK

## License

MIT
