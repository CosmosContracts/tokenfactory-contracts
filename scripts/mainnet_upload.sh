#!/bin/bash
# Uploads the contracts to Mainnet & returns their code_ids
#
# Usage: sh scripts/mainnet_upload.sh
#

KEY="reece"
CHAINID="juno-1"
JUNOD_HOME="$HOME/.juno"
KEYALGO="secp256k1"
KEYRING="os"

export JUNOD_NODE="https://rpc.juno.strange.love:443"
export JUNOD_COMMAND_ARGS="--gas=auto -y --from $KEY --broadcast-mode block --output json --chain-id=$CHAINID --gas-prices=0.005ujuno --gas-adjustment=1.4 --home "$JUNOD_HOME" --keyring-backend $KEYRING"

middleware_tx=$(junod tx wasm store artifacts/tokenfactory_core.wasm $JUNOD_COMMAND_ARGS | jq -r .txhash)
middleware_code_id=$(junod q tx $middleware_tx --output=json | jq -r '.logs[0].events[] | select(.type=="store_code")' | jq -r .attributes[1].value) && echo "Middleware code_id: $middleware_code_id"

migrate_tx=$(junod tx wasm store artifacts/migrate.wasm $JUNOD_COMMAND_ARGS | jq -r .txhash) # && echo "Migrate tx: $migrate_tx"
migrate_code_id=$(junod q tx $migrate_tx --output=json | jq -r '.logs[].events[] | select(.type=="store_code")' | jq -r .attributes[1].value) && echo "Migrate code_id: $migrate_code_id"