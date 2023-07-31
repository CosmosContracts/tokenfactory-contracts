#!/bin/bash
# Uploads the contracts & returns their code_ids
#
# Usage: sh scripts/testnet_upload.sh
#

KEY="validator"
CHAINID="uni-6"
JUNOD_HOME="$HOME/.juno"
KEYALGO="secp256k1"
KEYRING="os"

export JUNOD_NODE="https://uni-rpc.reece.sh:443"
export JUNOD_COMMAND_ARGS="--gas=auto -y --from $KEY --broadcast-mode=sync --output json --chain-id=$CHAINID --gas-prices=0.003ujunox --gas-adjustment=1.5 --home "$JUNOD_HOME" --keyring-backend $KEYRING"

middleware_tx=$(junod tx wasm store artifacts/tokenfactory_core.wasm $JUNOD_COMMAND_ARGS | jq -r .txhash)
middleware_code_id=$(junod q tx $middleware_tx --output=json | jq -r '.logs[0].events[] | select(.type=="store_code")' | jq -r .attributes[1].value) && echo "Middleware code_id: $middleware_code_id"

migrate_tx=$(junod tx wasm store artifacts/migrate.wasm $JUNOD_COMMAND_ARGS | jq -r .txhash) # && echo "Migrate tx: $migrate_tx"
migrate_code_id=$(junod q tx $migrate_tx --output=json | jq -r '.logs[].events[] | select(.type=="store_code")' | jq -r .attributes[1].value) && echo "Migrate code_id: $migrate_code_id"