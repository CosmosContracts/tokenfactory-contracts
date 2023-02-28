# CW20 & Native Migrations to TokenFactory

- Send in an old CW20 token and receive the new token-factory native token.
- Deposit a denomination (IBC, another factory token, or JUNO) and receive the new token-factory native token.

This requires the [TokenFactory Core Contract](https://github.com/CosmosContracts/tokenfactory-contracts). It allows for multiple contracts to mint for a single native token-factory denomination. Once this contract is initialized and the admin token set to the middleware, you are then ready to get this contract set up.

View documentation on how to set up your TokenFactory token and metadata: TODO.

---

Steps to begin:

- Initialize the core-middleware & token factory denomination *(The above link has a guide)*
- Initialize this migration contract with the following message

```json
// Example: e2e/migrate/test_e2e.sh upload_cw20mint
{    
    "cw20_token_address":"juno1MyCW20ContractTokenAddress",
    "contract_minter_address":"juno1CoreMiddlewareContractAddress",
    "tf_denom":"factory/juno1.../token"
}

// or
{    
    "burn_denom":"ibc/ABCDEFGHIJKLMNOPQ",
    "contract_minter_address":"juno1CoreMiddlewareContractAddress",
    "tf_denom":"factory/juno1.../token"
}
```

**NOTE** burn_denom can also be a `factory/juno1.../token` or `ujuno` itself

- On the core-middleware contract, add this address to the minter whitelist

```json
// Core middleware contract
{
    "add_whitelist":{"addresses":["juno1MigrateContractAddress"]}
}
```

Your contract is now ready for users to deposit a CW20 or native token. In return, the Middleware contract will mint the new token-factory native token for them at a 1:1 ratio!

---

## Other Ideas

<https://hackmd.io/@reecepbcups/cw20-to-tokenfactory>

- CW20 standard contract with a migrate function (BankSend the factory denominations to the contract, upload new CW20-tf-migrate if total CW20 supply <= held tokenfactory, convert all to the new denom)
^ Will we hit a gas limit issue? since Juno is only 10m per block

- DAODAO native converts with VoteModule / CW20 wrappers (on the usage of the token in DAODAO, it burns the CW20 and gives the user the native)