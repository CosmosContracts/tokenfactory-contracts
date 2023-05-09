package test

// NEW CONTRACTS HERE:
// cw20Msg := fmt.Sprintf(`{"name":"test","symbol":"aaaa","decimals":6,"initial_balances":[{"address":"%s","amount":"100"}]}`, uaddr)
// _, cw20ContractAddr := helpers.SetupContract(t, ctx, juno, user.KeyName, "../../base_artifacts/cw20_base.wasm", cw20Msg)

// Setup the migration contract (convert a cw20 to a native denom (ex: ibc, native, or factory))
// migrateCW20Msg := fmt.Sprintf(`{"cw20_token_address":"%s","contract_minter_address":"%s","tf_denom":"%s"}`, cw20ContractAddr, tfCoreContractAddr, tfDenom)
// _, cw20MigrateContractAddr := helpers.SetupContract(t, ctx, juno, user.KeyName, "../../artifacts/migrate.wasm", migrateCW20Msg)

// // Allow the Migration contract to mint through the Tokenfactory Core contract
// msg := fmt.Sprintf(`{"add_whitelist":{"addresses":["%s"]}}`, cw20MigrateContractAddr)
// if _, err := juno.ExecuteContract(ctx, user.KeyName, tfCoreContractAddr, msg); err != nil {
// 	t.Fatal(err)
// }

// // Ensure the contract config data is set correctly.
// res := GetContractConfig(t, ctx, juno, tfCoreContractAddr, uaddr)
// // t.Log("v.Data", v.Data)
// assert.Equal(t, res.Data.AllowedMintAddresses[0], cw20MigrateContractAddr)
// assert.Equal(t, res.Data.Denoms[0], tfDenom)

// // remove whitelist and ensure no one can mint
// msg = fmt.Sprintf(`{"remove_whitelist":{"addresses":["%s"]}}`, cw20MigrateContractAddr)
// if _, err := juno.ExecuteContract(ctx, user.KeyName, tfCoreContractAddr, msg); err != nil {
// 	t.Fatal(err)
// }
// // get config and ensure they are gone
// res = GetContractConfig(t, ctx, juno, tfCoreContractAddr, uaddr)
// assert.Equal(t, len(res.Data.AllowedMintAddresses), 0)
