source ./e2e/migrate/helpers.sh

CONTAINER_NAME="tokenfactory_migrate_native_test"
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
# === Contract Uploads ===
# ========================
function upload_tokenfactory_core {
    echo "Storing contract..."
    create_denom
    UPLOAD=$($BINARY tx wasm store /tokenfactory_core.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $UPLOAD
    BASE_CODE_ID=$($BINARY q tx $UPLOAD --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $BASE_CODE_ID"

    # == INSTANTIATE ==
    # no allowed_mint_addresses initially until we make the cw20burnmint, then we will add it as the admin of this contract via an execute
    PAYLOAD=$(printf '{"allowed_mint_addresses":[],"denoms":["%s"]}' $FULL_DENOM) && echo $PAYLOAD
    TX_HASH=$($BINARY tx wasm instantiate "$BASE_CODE_ID" "$PAYLOAD" --label "tf-middlware" $JUNOD_COMMAND_ARGS --admin "$KEY_ADDR" | jq -r '.txhash') && echo $TX_HASH

    export TF_CONTRACT=$($BINARY query tx $TX_HASH --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "TF_CONTRACT: $TF_CONTRACT"    
}

function upload_nativemigrate { # must run after uploading the tokenfactory core
    echo "Storing contract..."
    # its from the root of the docker container
    UPLOAD=$($BINARY tx wasm store /native_migrate.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $UPLOAD
    BASE_CODE_ID=$($BINARY q tx $UPLOAD --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $BASE_CODE_ID"

    # mode: balance or mint. If mint, contract_minter_address is required
    PAYLOAD=$(printf '{"burn_denom":"%s","contract_minter_address":"%s","tf_denom":"%s"}' $DENOM $TF_CONTRACT $FULL_DENOM) && echo $PAYLOAD
    TX_HASH=$($BINARY tx wasm instantiate "$BASE_CODE_ID" "$PAYLOAD" --label "cw20burnmint" $JUNOD_COMMAND_ARGS --admin "$KEY_ADDR" | jq -r '.txhash') && echo $TX_HASH

    export NATIVE_MIGRATE=$($BINARY query tx $TX_HASH --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "NATIVE_MIGRATE: $NATIVE_MIGRATE"

    # execute on the tokenfactory core as the admin to set this CW20_BURN contract to be allowed to mint on its behalf
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
upload_tokenfactory_core

# Our contracts
upload_nativemigrate
transfer_denom_to_middleware_contract

function test_native_contract_mint {
    # get balance of the $KEY_ADDR
    # 0 initially
    v=$($BINARY q bank balances $KEY_ADDR --denom $FULL_DENOM --output json | jq -r .amount) && echo $v
    ASSERT_EQUAL "$v" "0"

    # send 5 via the cw20base contract
    # sendCw20Msg $CW20_BURN "5"
    # execute Convert{} on the native_migrate contract with --amount 5    
    TX_HASH=$($BINARY tx wasm execute "$NATIVE_MIGRATE" '{"convert":{}}' --amount 5$DENOM $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $TX_HASH

    # should now be 5
    v=$($BINARY q bank balances $KEY_ADDR --denom $FULL_DENOM --output json | jq -r .amount) && echo $v
    ASSERT_EQUAL "$v" "5"

    # ensure the contract does not hold funds for any reason
    v=$($BINARY q bank balances $NATIVE_MIGRATE --denom $FULL_DENOM --output json | jq -r .amount) && echo $v
    ASSERT_EQUAL "$v" "0"
}

test_balance_contract

exit $FINAL_STATUS_CODE # from helpers.sh
# then you can continue to use your TF_CONTRACT for other applications :D