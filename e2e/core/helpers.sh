### Docker ###
# ===================
# === Docker Init ===
# ===================
function stop_docker {
    docker kill $CONTAINER_NAME
    docker rm $CONTAINER_NAME
    docker volume rm --force junod_data
}

function start_docker {
    IMAGE_TAG=${2:-"13.0.0-beta"}
    BLOCK_GAS_LIMIT=${GAS_LIMIT:-10000000} # mirrors mainnet

    echo "Building $IMAGE_TAG"
    echo "Configured Block Gas Limit: $BLOCK_GAS_LIMIT"

    stop_docker    

    # run junod docker
    docker run --rm -d --name $CONTAINER_NAME \
        -e STAKE_TOKEN=$DENOM \
        -e GAS_LIMIT="$GAS_LIMIT" \
        -e UNSAFE_CORS=true \
        -e TIMEOUT_COMMIT="500ms" \
        -p 1317:1317 -p 26656:26656 -p 26657:26657 \
        --mount type=volume,source=junod_data,target=/root \
        ghcr.io/cosmoscontracts/juno:$IMAGE_TAG /opt/setup_and_run.sh $KEY_ADDR    
}

function compile_and_copy {    
    # compile vaults contract here
    docker run --rm -v "$(pwd)":/code \
      --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
      --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
      cosmwasm/rust-optimizer:0.12.11

    # copy wasm to docker container
    docker cp ./artifacts/. $CONTAINER_NAME:/
}


## User Accounts
function add_accounts {
    # provision juno default user i.e. juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl
    echo "decorate bright ozone fork gallery riot bus exhaust worth way bone indoor calm squirrel merry zero scheme cotton until shop any excess stage laundry" | $BINARY keys add test-user --recover --keyring-backend test
    # juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk
    echo "wealth flavor believe regret funny network recall kiss grape useless pepper cram hint member few certain unveil rather brick bargain curious require crowd raise" | $BINARY keys add other-user --recover --keyring-backend test
    # juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y
    echo "clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose" | $BINARY keys add user3 --recover --keyring-backend test

    # send some 10 junox funds to the users
    $BINARY tx bank send test-user juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk 10000000ujunox $JUNOD_COMMAND_ARGS
    $BINARY tx bank send test-user juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y 100000ujunox $JUNOD_COMMAND_ARGS

    # check funds where sent
    # other_balance=$($BINARY q bank balances juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk --output json)
    # ASSERT_EQUAL "$other_balance" '{"balances":[{"denom":"ujunox","amount":"10000000"}],"pagination":{"next_key":null,"total":"0"}}'
}

# ========================
# === Helper Functions ===
# ========================
function health_status {
    # validator addr
    VALIDATOR_ADDR=$($BINARY keys show validator --address) && echo "Validator address: $VALIDATOR_ADDR"

    BALANCE_1=$($BINARY q bank balances $VALIDATOR_ADDR) && echo "Pre-store balance: $BALANCE_1"

    echo "Address to deploy contracts: $KEY_ADDR"
    echo "JUNOD_COMMAND_ARGS: $JUNOD_COMMAND_ARGS"
}

function query_contract {
    $BINARY query wasm contract-state smart $1 $2 --output json
}

function wasm_cmd {
    CONTRACT=$1
    MESSAGE=$2
    FUNDS=$3
    SHOW_LOG=${4:dont_show}
    ARGS=${5:-$JUNOD_COMMAND_ARGS}
    echo "EXECUTE $MESSAGE on $CONTRACT"

    # if length of funds is 0, then no funds are sent
    if [ -z "$FUNDS" ]; then
        FUNDS=""
    else
        FUNDS="--amount $FUNDS"
        echo "FUNDS: $FUNDS"
    fi
    
    # echo "ARGS: $ARGS"

    tx_hash=$($BINARY tx wasm execute $CONTRACT $MESSAGE $FUNDS $ARGS | jq -r '.txhash')
    export GAS_USED=$($BINARY query tx $tx_hash --output json | jq -r '.gas_used')    
    export CMD_LOG=$($BINARY query tx $tx_hash --output json | jq -r '.raw_log')    
    if [ "$SHOW_LOG" == "show_log" ]; then
        echo -e "\nGAS_USED: $GAS_USED"
        echo -e "raw_log: $CMD_LOG\n================================\n"
    fi    
}

# CW721
function mint_cw721 {
    CONTRACT_ADDR=$1
    TOKEN_ID=$2
    OWNER=$3
    TOKEN_URI=$4
    EXECUTED_MINT_JSON=`printf '{"mint":{"token_id":"%s","owner":"%s","token_uri":"%s"}}' $TOKEN_ID $OWNER $TOKEN_URI`
    TXMINT=$($BINARY tx wasm execute "$CONTRACT_ADDR" "$EXECUTED_MINT_JSON" $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $TXMINT
}

function send_nft_to_listing {
    INTERATING_CONTRACT=$1
    CW721_CONTRACT_ADDR=$2
    TOKEN_ID=$3
    LISTING_ID=$4
    
    NFT_LISTING_BASE64=`printf '{"add_to_listing_cw721":{"listing_id":"%s"}}' $LISTING_ID | base64 -w 0`    
    SEND_NFT_JSON=`printf '{"send_nft":{"contract":"%s","token_id":"%s","msg":"%s"}}' $INTERATING_CONTRACT $TOKEN_ID $NFT_LISTING_BASE64`        

    wasm_cmd $CW721_CONTRACT_ADDR "$SEND_NFT_JSON" "" show_log
}

# CW20 Tokens
function send_cw20_to_listing {
    INTERATING_CONTRACT=$1
    CW20_CONTRACT_ADDR=$2
    AMOUNT=$3
    LISTING_ID=$4
    
    LISTING_BASE64=`printf '{"add_funds_to_sale_cw20":{"listing_id":"%s"}}' $LISTING_ID | base64 -w 0`               
    SEND_TOKEN_JSON=`printf '{"send":{"contract":"%s","amount":"%s","msg":"%s"}}' $INTERATING_CONTRACT $AMOUNT $LISTING_BASE64`        

    wasm_cmd $CW20_CONTRACT_ADDR "$SEND_TOKEN_JSON" "" show_log
}

# ===============
# === ASSERTS ===
# ===============
FINAL_STATUS_CODE=0

function ASSERT_EQUAL {
    # For logs, put in quotes. 
    # If $1 is from JQ, ensure its -r and don't put in quotes
    if [ "$1" != "$2" ]; then        
        echo "ERROR: $1 != $2" 1>&2
        FINAL_STATUS_CODE=1 
    else
        echo "SUCCESS"
    fi
}
function ASSERT_CONTAINS {
    if [[ "$1" != *"$2"* ]]; then
        echo "ERROR: $1 does not contain $2" 1>&2        
        FINAL_STATUS_CODE=1 
    else
        echo "SUCCESS"
    fi
}