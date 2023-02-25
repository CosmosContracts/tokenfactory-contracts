# Test script for Juno Smart Contracts (By @Reecepbcups)
# ./github/workflows/e2e.yml
#
# sh ./e2e/test_e2e.sh
#
# NOTES: anytime you use jq, use `jq -rc` for ASSERT_* functions (-c removes format, -r is raw to remove \" quotes)

# get functions from helpers file 
# -> query_contract, wasm_cmd, mint_cw721, send_nft_to_listing, send_cw20_to_listing
source ./e2e/migrate/helpers.sh

CONTAINER_NAME="tokenfactory_migratecw20_test"
BINARY="docker exec -i $CONTAINER_NAME junod"
DENOM='ujunox'
JUNOD_CHAIN_ID='testing'
JUNOD_NODE='http://localhost:26657/'
# globalfee will break this in the future
TX_FLAGS="--gas-prices 0.1$DENOM --gas-prices="0ujunox" --gas 5000000 -y -b block --chain-id $JUNOD_CHAIN_ID --node $JUNOD_NODE --output json"
export JUNOD_COMMAND_ARGS="$TX_FLAGS --from test-user"
export KEY_ADDR="juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl"

MAIN_REPO_RAW_ARTIFACTS="https://github.com/Reecepbcups/tokenfactory-core-contract/raw/main/artifacts"

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
function upload_cw20_base {
    UPLOAD=$($BINARY tx wasm store /cw20_base.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $UPLOAD
    BASE_CODE_ID=$($BINARY q tx $UPLOAD --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $BASE_CODE_ID"

    CW20_TX_INIT=$($BINARY tx wasm instantiate "$BASE_CODE_ID" '{"name":"test","symbol":"aaaa","decimals":6,"initial_balances":[{"address":"juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl","amount":"100"}]}' --label "juno-cw20" $JUNOD_COMMAND_ARGS -y --admin $KEY_ADDR | jq -r '.txhash') && echo $CW20_TX_INIT
    export CW20_ADDR=$($BINARY query tx $CW20_TX_INIT --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "$CW20_ADDR"
}
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

function upload_cw20mint { # must run after uploading the tokenfactory core
    echo "Storing contract..."
    # its from the root of the docker container
    UPLOAD=$($BINARY tx wasm store /cw20_migrate.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $UPLOAD
    BASE_CODE_ID=$($BINARY q tx $UPLOAD --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $BASE_CODE_ID"

    # mode: balance or mint. If mint, contract_minter_address is required
    PAYLOAD=$(printf '{"mode":"mint","cw20_token_address":"%s","contract_minter_address":"%s","tf_denom":"%s"}' $CW20_ADDR $TF_CONTRACT $FULL_DENOM) && echo $PAYLOAD
    TX_HASH=$($BINARY tx wasm instantiate "$BASE_CODE_ID" "$PAYLOAD" --label "cw20burnmint" $JUNOD_COMMAND_ARGS --admin "$KEY_ADDR" | jq -r '.txhash') && echo $TX_HASH

    export CW20_BURN=$($BINARY query tx $TX_HASH --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "CW20_BURN: $CW20_BURN"

    # execute on the tokenfactory core as the admin to set this CW20_BURN contract to be allowed to mint on its behalf
    PAYLOAD=$(printf '{"add_whitelist":{"addresses":["%s"]}}' $CW20_BURN) && echo $PAYLOAD
    TX_HASH=$($BINARY tx wasm execute "$TF_CONTRACT" "$PAYLOAD" $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $TX_HASH

    # query the contract to see if it was added    
    v=$($BINARY query wasm contract-state smart $TF_CONTRACT '{"get_config":{}}' --output json | jq .data) && echo $v
    ASSERT_EQUAL "$(echo $v | jq -r .allowed_mint_addresses[0])" "$CW20_BURN"
    ASSERT_EQUAL "$(echo $v | jq -r .denoms[0])" "$FULL_DENOM"
    # the cw20burnmint address can now mint tokens from the TF_CONTRACT
}
function upload_cw20balance { # this does not use the middleware tokenfactory core. Just takes 
    echo "Storing contract..."
    # its from the root of the docker container
    UPLOAD=$($BINARY tx wasm store /cw20_migrate.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $UPLOAD
    BASE_CODE_ID=$($BINARY q tx $UPLOAD --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $BASE_CODE_ID"

    # mode: balance or mint.
    PAYLOAD=$(printf '{"mode":"balance","cw20_token_address":"%s","tf_denom":"%s"}' $CW20_ADDR $FULL_DENOM) && echo $PAYLOAD
    TX_HASH=$($BINARY tx wasm instantiate "$BASE_CODE_ID" "$PAYLOAD" --label "cw20burnbalance" $JUNOD_COMMAND_ARGS --admin "$KEY_ADDR" | jq -r '.txhash') && echo $TX_HASH

    export CW20_BALANCE=$($BINARY query tx $TX_HASH --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "CW20_BALANCE: $CW20_BALANCE"

    # query
    v=$($BINARY query wasm contract-state smart $CW20_BALANCE '{"get_config":{}}' --output json | jq -r .data) && echo $v
    ASSERT_EQUAL "$(echo $v | jq -r .mode)" "balance"
    
    TOKENS_AMOUNT=50

    # mint some tokenfactory tokens and send to CW20_BALANCE
    $BINARY tx tokenfactory mint $TOKENS_AMOUNT$FULL_DENOM $JUNOD_COMMAND_ARGS
    $BINARY tx bank send $KEY_ADDR $CW20_BALANCE $TOKENS_AMOUNT$FULL_DENOM $JUNOD_COMMAND_ARGS

    # check CW20_BALANCE (50)
    v=$($BINARY q bank balances $CW20_BALANCE --output json | jq -r .balances) && echo $v
    ASSERT_EQUAL "$(echo $v | jq -r .[0].amount)" "$TOKENS_AMOUNT"
}

# === COPY ALL ABOVE TO SET ENVIROMENT UP LOCALLY ====

# =============
# === LOGIC ===
# =============

start_docker
download_latest
compile_and_copy # the compile takes time for the docker container to start up
add_accounts
# health check
health_status
# upload base contracts
upload_cw20_base
upload_tokenfactory_core

# Our contracts
upload_cw20balance # MUST CALL THIS FIRST BEFORE WE TRANSFER THE TOKENFACTORY TOKEN TO THE MINT MODULE

transfer_denom_to_middleware_contract
upload_cw20mint


# we are going to send some balance from the CW20 to the cw20burnmint address and ensure they get the tokens in return
function sendCw20Msg() {
    THIS_CONTRACT=$1
    AMOUNT=$2

    BASE64_MSG=$(echo -n "{"receive":{}}" | base64)
    export EXECUTED_MINT_JSON=`printf '{"send":{"contract":"%s","amount":"%s","msg":"%s"}}' $THIS_CONTRACT "$AMOUNT" $BASE64_MSG` && echo $EXECUTED_MINT_JSON

    # Base cw20 contract
    TX=$($BINARY tx wasm execute "$CW20_ADDR" "$EXECUTED_MINT_JSON" $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $TX
    # junod tx wasm execute "$CW20_ADDR" `printf '{"send":{"contract":"%s","amount":"5","msg":"e3JlZGVlbTp7fX0="}}' $BURN_ADDR` $JUNOD_COMMAND_ARGS
}

function test_mint_contract {
    # get balance of the $KEY_ADDR
    # 0 initially
    v=$($BINARY q bank balances $KEY_ADDR --denom $FULL_DENOM --output json | jq -r .amount) && echo $v
    ASSERT_EQUAL "$v" "0"

    # send 5 via the cw20base contract
    sendCw20Msg $CW20_BURN "5"

    # should now be 5
    v=$($BINARY q bank balances $KEY_ADDR --denom $FULL_DENOM --output json | jq -r .amount) && echo $v
    ASSERT_EQUAL "$v" "5"

    # 0 since this does not hold balance
    v=$($BINARY q bank balances $CW20_BURN --denom $FULL_DENOM --output json | jq -r .amount) && echo $v
    ASSERT_EQUAL "$v" "0"
}

function test_balance_contract {
    # should be 5
    v=$($BINARY q bank balances $KEY_ADDR --denom $FULL_DENOM --output json | jq -r .amount) && echo $v
    ASSERT_EQUAL "$v" "5"

    sendCw20Msg $CW20_BALANCE "2"

    # should be 7    
    v=$($BINARY q bank balances $KEY_ADDR --denom $FULL_DENOM --output json | jq -r .amount) && echo $v
    ASSERT_EQUAL "$v" "7"

    # ensure the balance of the cw20_balance contract went down 2
    v=$($BINARY q bank balances $CW20_BALANCE --denom $FULL_DENOM --output json | jq -r .amount) && echo $v
    ASSERT_EQUAL "$v" "$(( $TOKENS_AMOUNT - 2 ))"
}


test_mint_contract
test_balance_contract

exit $FINAL_STATUS_CODE # from helpers.sh
# then you can continue to use your TF_CONTRACT for other applications :D