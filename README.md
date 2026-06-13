# TrustPay - Decentralized Escrow on Stellar

## Problem
Freelancers fear clients won't pay, and clients fear freelancers won't deliver the work.

## Solution
A decentralized escrow smart contract that securely locks client deposits and automatically releases funds only upon job completion, featuring deadline tracking and admin dispute resolution.

## Why Stellar
Soroban's native `require_auth()` capability makes multi-signature authorization and role-based access control incredibly secure and straightforward for escrow payment flows.

## Target User
Freelancers, independent contractors, and clients hiring remote talent.

## Live Demo
- Network: Stellar Testnet
- **Contract ID**: `CBSRJTFRXCXNZSJN57UCTZ2B5GFXS7USYFR5PTDRCGV3B7DJCXP5UYW4`
- **Transaction (Successful Escrow Release)**: https://stellar.expert/explorer/testnet/tx/380a3a9b2b6032c24396db284b7ada475ec00a068e44d86a6d215c962ea8b879

## How to Run
1. Clone: `git clone https://github.com/Theace-vip/NguyenThieuBao_TrustPay_DecentralizedEscrowonStellar.git`
2. Build: `cd contracts/trustpay && stellar contract build`
3. Test: `cargo test`
4. Deploy: `stellar contract deploy --wasm target/wasm32-unknown-unknown/release/trustpay.wasm --source-account admin --network testnet`
5. Frontend: `cd frontend && npx serve .`

## Tech Stack
- Smart Contract: Rust / Soroban SDK v25
- Frontend: HTML / JavaScript / @stellar/stellar-sdk
- Wallet: Freighter
- Network: Stellar Testnet

## Team
- [Nguyễn Thiếu Bảo] | [bao25122005@example.com] | Ho Chi Minh City University of Technology (HUTECH)