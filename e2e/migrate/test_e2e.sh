# Test script for Juno Smart Contracts (By @Reecepbcups)
# ./github/workflows/e2e.yml
#
# sh ./e2e/test_e2e.sh
#
# NOTES: anytime you use jq, use `jq -rc` for ASSERT_* functions (-c removes format, -r is raw to remove \" quotes)
#
#
source ./e2e/migrate/helpers.sh

CONTAINER_NAME="tokenfactory_migrate_test"
BINARY="docker exec -i $CONTAINER_NAME junod"
DENOM='ujunox'
JUNOD_CHAIN_ID='testing'
JUNOD_NODE='http://localhost:26657/'
# globalfee will break this in the future
TX_FLAGS="--gas-prices 0.1$DENOM --gas-prices="0ujunox" --gas 5000000 -y -b block --chain-id $JUNOD_CHAIN_ID --node $JUNOD_NODE --output json"
export JUNOD_COMMAND_ARGS="$TX_FLAGS --from test-user"
export KEY_ADDR="juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl"

MAIN_REPO_RAW_ARTIFACTS="https://github.com/CosmosContracts/tokenfactory-contracts/raw/main/artifacts"

function create_denom {
    RANDOM_STRING=$(cat /dev/urandom | tr -dc 'a-zA-Z' | fold -w 6 | head -n 1)

    $BINARY tx tokenfactory create-denom $RANDOM_STRING $JUNOD_COMMAND_ARGS    
    export FULL_DENOM="factory/$KEY_ADDR/$RANDOM_STRING" && echo $FULL_DENOM
}

function transfer_denom_to_middleware_contract {
    # transfer admin to the contract from the user (this way the contract can mint factory denoms)
    $BINARY tx tokenfactory change-admin $FULL_DENOM $TF_CONTRACT $JUNOD_COMMAND_ARGS
    $BINARY q tokenfactory denom-authority-metadata $FULL_DENOM # admin is the TF_CONTRACT
}

function download_latest {
    # download latest core contract from public repo, gets uploaded to the docker container
    wget -O e2e/migrate/tokenfactory_core.wasm "$MAIN_REPO_RAW_ARTIFACTS/tokenfactory_core.wasm"
}

# ========================
# ===   Core Uploads   ===
# ========================
function upload_cw20_base {
    UPLOAD=$($BINARY tx wasm store /cw20_base.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $UPLOAD
    BASE_CODE_ID=$($BINARY q tx $UPLOAD --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $BASE_CODE_ID"

    CW20_TX_INIT=$($BINARY tx wasm instantiate "$BASE_CODE_ID" '{"name":"test","symbol":"aaaa","decimals":6,"initial_balances":[{"address":"juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl","amount":"100"}]}' --label "juno-cw20" $JUNOD_COMMAND_ARGS -y --admin $KEY_ADDR | jq -r '.txhash') && echo $CW20_TX_INIT
    export CW20_ADDR=$($BINARY query tx $CW20_TX_INIT --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "$CW20_ADDR"
}

function upload_tokenfactory_core {
    echo "Storing contract..."    
    create_denom # must run here
    UPLOAD=$($BINARY tx wasm store /tokenfactory_core.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $UPLOAD
    BASE_CODE_ID=$($BINARY q tx $UPLOAD --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $BASE_CODE_ID"

    # == INSTANTIATE ==
    # no allowed_mint_addresses initially until we make the cw20burnmint, then we will add it as the admin of this contract via an execute
    PAYLOAD=$(printf '{"allowed_mint_addresses":[],"denoms":["%s"]}' $FULL_DENOM) && echo $PAYLOAD
    TX_HASH=$($BINARY tx wasm instantiate "$BASE_CODE_ID" "$PAYLOAD" --label "tf-middlware" $JUNOD_COMMAND_ARGS --admin "$KEY_ADDR" | jq -r '.txhash') && echo $TX_HASH

    export TF_CONTRACT=$($BINARY query tx $TX_HASH --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "TF_CONTRACT: $TF_CONTRACT"    
}

# ========================
# === Contract Uploads ===
# ========================
function upload_cw20mint { # must run after uploading the tokenfactory core
    echo "Storing contract..."
    # its from the root of the docker container
    UPLOAD=$($BINARY tx wasm store /migrate.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $UPLOAD
    BASE_CODE_ID=$($BINARY q tx $UPLOAD --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $BASE_CODE_ID"

    # mode: balance or mint. If mint, contract_minter_address is required
    PAYLOAD=$(printf '{"cw20_token_address":"%s","contract_minter_address":"%s","tf_denom":"%s"}' $CW20_ADDR $TF_CONTRACT $FULL_DENOM) && echo $PAYLOAD
    TX_HASH=$($BINARY tx wasm instantiate "$BASE_CODE_ID" "$PAYLOAD" --label "cw20burnmint" $JUNOD_COMMAND_ARGS --admin "$KEY_ADDR" | jq -r '.txhash') && echo $TX_HASH

    export CW20_MIGRATE=$($BINARY query tx $TX_HASH --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "CW20_MIGRATE: $CW20_MIGRATE"

    # execute on the tokenfactory core as the admin to set this CW20_MIGRATE contract to be allowed to mint on its behalf
    PAYLOAD=$(printf '{"add_whitelist":{"addresses":["%s"]}}' $CW20_MIGRATE) && echo $PAYLOAD
    TX_HASH=$($BINARY tx wasm execute "$TF_CONTRACT" "$PAYLOAD" $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $TX_HASH

    # query the contract to see if it was added    
    v=$($BINARY query wasm contract-state smart $TF_CONTRACT '{"get_config":{}}' --output json | jq .data) && echo $v
    ASSERT_EQUAL "$(echo $v | jq -r .allowed_mint_addresses[0])" "$CW20_MIGRATE"
    ASSERT_EQUAL "$(echo $v | jq -r .denoms[0])" "$FULL_DENOM"
    # the cw20burnmint address can now mint tokens from the TF_CONTRACT
}

function upload_nativemigrate { # must run after uploading the tokenfactory core
    echo "Storing contract..."
    # its from the root of the docker container
    UPLOAD=$($BINARY tx wasm store /migrate.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $UPLOAD
    BASE_CODE_ID=$($BINARY q tx $UPLOAD --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $BASE_CODE_ID"

    # mode: balance or mint. If mint, contract_minter_address is required
    PAYLOAD=$(printf '{"burn_denom":"%s","contract_minter_address":"%s","tf_denom":"%s"}' "$DENOM" $TF_CONTRACT $FULL_DENOM) && echo $PAYLOAD
    TX_HASH=$($BINARY tx wasm instantiate "$BASE_CODE_ID" "$PAYLOAD" --label "cw20burnmint" $JUNOD_COMMAND_ARGS --admin "$KEY_ADDR" | jq -r '.txhash') && echo $TX_HASH

    export NATIVE_MIGRATE=$($BINARY query tx $TX_HASH --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "NATIVE_MIGRATE: $NATIVE_MIGRATE"
    
    PAYLOAD=$(printf '{"add_whitelist":{"addresses":["%s"]}}' $NATIVE_MIGRATE) && echo $PAYLOAD
    TX_HASH=$($BINARY tx wasm execute "$TF_CONTRACT" "$PAYLOAD" $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $TX_HASH

    # query the contract to see if it was added    
    v=$($BINARY query wasm contract-state smart $TF_CONTRACT '{"get_config":{}}' --output json | jq .data) && echo $v
    ASSERT_EQUAL "$(echo $v | jq -r .allowed_mint_addresses[0])" "$NATIVE_MIGRATE"
    ASSERT_EQUAL "$(echo $v | jq -r .denoms[0])" "$FULL_DENOM"
}

# === COPY ALL ABOVE TO SET ENVIROMENT UP LOCALLY ====

# =============
# === LOGIC ===
# =============

start_docker
download_latest
compile_and_copy # the compile takes time for the docker container to start up
add_accounts

# upload base contracts
upload_cw20_base

# cw20
upload_tokenfactory_core
transfer_denom_to_middleware_contract
upload_cw20mint
function test_cw20_contract {
    # get balance of the $KEY_ADDR
    # 0 initially
    v=$($BINARY q bank balances $KEY_ADDR --denom $FULL_DENOM --output json | jq -r .amount) && echo $v
    ASSERT_EQUAL "$v" "0"

    # send 5 via the cw20base contract
    send_cw20_msg $CW20_MIGRATE "5"

    # should now be 5
    v=$($BINARY q bank balances $KEY_ADDR --denom $FULL_DENOM --output json | jq -r .amount) && echo $v
    ASSERT_EQUAL "$v" "5"

    # 0 since this does not hold balance
    v=$($BINARY q bank balances $CW20_MIGRATE --denom $FULL_DENOM --output json | jq -r .amount) && echo $v
    ASSERT_EQUAL "$v" "0"
}
test_cw20_contract





# native
upload_tokenfactory_core
transfer_denom_to_middleware_contract
upload_nativemigrate
function test_native_contract {
    v=$($BINARY q bank balances $KEY_ADDR --denom $FULL_DENOM --output json | jq -r .amount) && echo $v
    ASSERT_EQUAL "$v" "0"

    # wasm execute on the NATIVE_MIGRATE contract
    TX=$($BINARY tx wasm execute "$NATIVE_MIGRATE" '{"convert":{}}' --amount 2$DENOM $JUNOD_COMMAND_ARGS) && echo $TX
    
    v=$($BINARY q bank balances $KEY_ADDR --denom $FULL_DENOM --output json | jq -r .amount) && echo $v
    ASSERT_EQUAL "$v" "2"

    # 0 since this does not hold balance
    v=$($BINARY q bank balances $NATIVE_MIGRATE --denom $FULL_DENOM --output json | jq -r .amount) && echo $v
    ASSERT_EQUAL "$v" "0"

    $BINARY q wasm contract-state smart $NATIVE_MIGRATE '{"get_config":{}}' --output json | jq .data
}
test_native_contract



exit $FINAL_STATUS_CODE # from helpers.sh