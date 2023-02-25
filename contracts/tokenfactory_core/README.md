# TokenFactory Core (middleware)

This is a contract to which you give the admin of your token denomination from the TokenFactory module. It then allows other contracts you control to mint tokens on this contract's behalf via a WasmMsg.

This makes it more flexible since multiple contracts can "mint" tokens on behalf of the contract admin :D

---

## Rust Dependency

Add the following to your `Cargo.toml` dependencies for a contract. Then view the Mint section of this document for how to implement it.

```toml
[dependencies]
tokenfactory-types = { git = "https://github.com/Reecepbcups/tokenfactory-core-contract" }
```

You can view an example of how to use this in the [example contract](./contracts/tf_example/) or see the [e2e test](./e2e/test_e2e.sh) for a full example in bash.

---

## Chain Setup

Mainnet Store Code: TBD

```sh
# update [key] here
FLAGS="--gas-prices="0.003ujuno" --gas auto -y -b block --chain-id juno-1 --node https://juno-rpc.reece.sh:443 --output json --from [key]"

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
use tokenfactory_types::msg::ExecuteMsg::Mint;

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
