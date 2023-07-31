# TokenFactory Core (middleware)

This is a contract to which you give the admin of your token denomination(s) from the TokenFactory module. Once this contract has that, it allows other contracts you/your DAO controls to mint tokens for your business logic needs (via a WasmMsg).

This makes it more flexible since multiple contracts can "mint" tokens on behalf of the contract admin :D

## Example Use Case

$RAC has slots & dice contracts. If every game they want to mint 1 RAC for you for playing, both slots and dice would need to be admin of the token-factory token to mint
With this core contract, a single contract is an admin, then the DAO can whitelist both the dice and slots address to mint tokens on its behalf.

This way to mint $RAC natively, the dice contract would WasmMsg a mint to the core contract, to then give the user that token.

---

## Rust Dependency

Add the following to your `Cargo.toml` dependencies for a contract. Then view the Mint section of this document for how to implement it.

```toml
[dependencies]
tokenfactory-types = { git = "https://github.com/CosmosContracts/tokenfactory-contracts" }
```

You can view an example of how to use this in the [example contract](https://github.com/CosmosContracts/tokenfactory-contracts/tree/main/contracts/tf_example/src) or see the [e2e test](https://github.com/CosmosContracts/tokenfactory-contracts/blob/main/e2e/core/test_e2e.sh) for a full example in bash.

---

## Chain Setup

Mainnet Store Code: TBD

```sh
# for uni-6 TESTNET
# update [key] here to be your local wallet's key or your wallet's address
FLAGS="--gas-prices 0.003ujuno --gas auto --gas-adjustment 1.3 --chain-id uni-6 --node https://juno-testnet-rpc.polkachu.com:443 --output json --from [key]"

# create a tokenfactory denomination via the CLI.
junod tx tokenfactory create-denom abcde $FLAGS
# factory/juno1......./abcde is your new denom


# upload this contract (skip if you use the mainnet code)
# junod tx wasm store artifacts/tokenfactory_core.wasm $FLAGS

# Initialize this contract
# You may want to set this as a normal admin initially before changing its admin to a DAO
junod tx wasm instantiate "###" {"allowed_mint_addresses":[],"denoms":["factory/juno1./abcde"]} --label "tf-middlware" --admin [key] $FLAGS
# Get the middleware contract address here

# Transfer ownership of the token to the contract
junod tx tokenfactory change-admin factory/juno1./abcde juno1middlewarecontract $FLAGS

# Ensure the juno1middlewarecontract now has the admin role
junod q tokenfactory denom-authority-metadata factory/juno1./abcde
```

## How To Contract Mint

You can then mint tokens via another contract using the following example

```rust
// msg.rs - mint on behalf of the core_factory_address
#[cw_serde]
pub enum ExecuteMsg {
    MintTokens {
        // You could save this in state.rs on initialize.
        core_tf_middleware_contract: String,
        denoms: Vec<Coin>,
        to_address: String,
    },
}

// contract.rs - execute

// Ensure you added the tokenfactory-types dependency
use juno::juno_tokenfactory_types::msg::ExecuteMsg::Mint;

ExecuteMsg::MintTokens {
    core_tf_middleware_contract,
    denoms,
    to_address,
} => {
    let payload = Mint {
        address: to_address,
        denom: denoms,
    };
    let wasm_msg = WasmMsg::Execute {
        contract_addr: core_tf_middleware_contract.to_string(),
        msg: to_binary(&payload)?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_attribute("method", "execute_mint_tokens_from_other_contract")
        .add_message(wasm_msg))
}
```
