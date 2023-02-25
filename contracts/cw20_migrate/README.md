# CW20 Migrations to TokenFactory

Deposit/Send in an old CW20 token and receive the new token-factory native token :D

Modes: Mint  or send from this contract's bank balance

## Mode: mint

This mode requires the [Tokenfactory Core Contract](https://github.com/CosmosContracts/tokenfactory-contracts). It allows for multiple contracts to mint for a single native token-factory denomination. Once this contract is initialized and the admin token set to the middleware, you are then ready to get this contract set up.

Steps to enable mint mode:

- Initialize the core-middleware & token factory denom *(above link has a guide)*
- Initialize this migration contract with the following message

```json
// Example: e2e/test_e2e.sh upload_cw20mint
{
    "mode":"mint",
    "cw20_token_address":"juno1MyCW20ContractTokenAddress",
    "contract_minter_address":"juno1CoreMiddlewareContractAddress",
    "tf_denom":"factory/juno1.../token"
}
```

- On the core-middleware contract, add this address to the minter whitelist

```json
// Core middleware contract
{
    "add_whitelist":{"addresses":["juno1MigrateContractAddress"]}
}
```

Your contract is now ready for users to deposit a CW20, and in return, the Middleware contract will mint the new token-factory native token for them!

## Mode: balance

In this mode, the contract does not require the core middleware contract. It will simply send the native token-factory denom from the contract's bank balance to the user. This could mean the contract runs out of funds. 

Steps to enable balance mode:

- Initialize this migration contract with the following message

```json
// Example: e2e/test_e2e.sh upload_cw20balance
{
    "mode":"balance",
    "cw20_token_address":"juno1MyCW20ContractTokenAddress",    
    "tf_denom":"factory/juno1.../token"
}
```

- Send token-factory funds to this newly created contract address

```sh
# mint tokens to the admin account via the CLI
junod tx tokenfactory mint 1000factory/juno1...addr.../abcde $FLAGS

# send those tokens to the balance migration contract
junod tx bank send [key] <$CW20_BALANCE_CONTRACT> 1000factory/juno1...addr.../abcde $FLAGS

# NOTE: You could have a whitelisted member of the core TF middleware mint tokens to this address from another contract / user if you so choose.
```

---

## Other Ideas

Will work on these after Juno v13 launch

<https://hackmd.io/@reecepbcups/cw20-to-tokenfactory>

- CW20 standard contract with a migrate function (bankSend the factory denom to the contract, upload new CW20-tf-migrate if total CW20 supply <= held tokenfactory, convert all to the new denom)
^ Will we hit a gas limit issue? since juno is only 10m per block

- IBC convert denoms, send to null address? since bank doesn't have burn

- DAODAO native converts with VoteModule / CW20 wrappers (on usage of the token in DAODAO, it burns the CW20 and gives the user the native)