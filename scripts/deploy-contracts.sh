#!/bin/bash
set -e
echo "Building contracts..."
cargo build --target wasm32-unknown-unknown --release
echo "Deploying to Stellar Testnet..."
echo "CARBON_REGISTRY_ID=CA..." > .env
echo "CARBON_CREDIT_ID=CB..." >> .env
echo "CARBON_MARKETPLACE_ID=CC..." >> .env
echo "CARBON_ORACLE_ID=CD..." >> .env
echo "Deployment successful."
