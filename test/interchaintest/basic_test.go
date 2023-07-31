package test

import (
	"fmt"
	"testing"

	"github.com/strangelove-ventures/interchaintest/v4"
	"github.com/strangelove-ventures/interchaintest/v4/chain/cosmos"
	"gotest.tools/assert"

	helpers "github.com/CosmosContracts/tokenfactory-contracts/helpers"
)

const CHAIN_PREFIX = "juno"

// This test ensures the basic contract logic works (bindings mostly & transfers)
// Actual contract logic checks are handled in the TestMigrateContract test
func TestBasicContract(t *testing.T) {
	t.Parallel()

	// Create chain factory with Juno
	chains := CreateBaseChain(t)
	ic, ctx, _, _ := BuildInitialChain(t, chains)
	juno := chains[0].(*cosmos.CosmosChain)

	// User Setup
	users := interchaintest.GetAndFundTestUsers(t, ctx, "default", int64(100_000_000), juno, juno)
	user := users[0]
	uaddr := user.Bech32Address(juno.Config().Bech32Prefix)
	user2 := users[1]
	uaddr2 := user2.Bech32Address(CHAIN_PREFIX)

	// Create token-factory denom
	tfDenom := helpers.CreateTokenFactoryDenom(t, ctx, juno, user, "testdenom")
	denomAdmin := helpers.GetTokenFactoryAdmin(t, ctx, juno, tfDenom)
	assert.Equal(t, uaddr, denomAdmin)

	// Setup TokenFactory Core contract (mints on your/daos behalf) where uaddr can mint for anyone
	tfCoreMsg := fmt.Sprintf(`{"allowed_mint_addresses":["%s"],"existing_denoms":["%s"]}`, uaddr, tfDenom)
	tfCoreCodeId, tfCoreContractAddr := helpers.SetupContract(t, ctx, juno, user.KeyName, "../../artifacts/tokenfactory_core.wasm", tfCoreMsg)

	assert.Assert(t, len(tfCoreContractAddr) > 0)
	res := GetContractConfig(t, ctx, juno, tfCoreContractAddr)
	assert.Assert(t, len(res.Data.AllowedMintAddresses) == 1)
	assert.Equal(t, res.Data.Denoms[0], tfDenom)

	tfCoreCodeId, err := juno.StoreContract(ctx, user.KeyName, "../../artifacts/tokenfactory_core.wasm")

	// transfer admin to the contract
	helpers.TransferTokenFactoryAdmin(t, ctx, juno, user, tfCoreContractAddr, tfDenom)
	denomAdmin = helpers.GetTokenFactoryAdmin(t, ctx, juno, tfDenom)
	assert.Equal(t, tfCoreContractAddr, denomAdmin)

	// Mint 100 tokens to user through the tfCore contract
	msg := fmt.Sprintf(`{"mint":{"address":"%s","denom":[{"denom":"%s","amount":"100"}]}}`, uaddr, tfDenom)
	juno.ExecuteContract(ctx, user.KeyName, tfCoreContractAddr, msg)

	// BALANCES
	CheckBalance(t, ctx, juno, uaddr, tfDenom, 100)
	// do the same thing but through the TF contract query
	balRes := GetCoreContractUserBalance(t, ctx, juno, tfCoreContractAddr, uaddr, tfDenom)
	assert.Equal(t, balRes.Data.Amount, "100")

	// Whitelist
	// Try to add user to contract whitelist again.
	msg = fmt.Sprintf(`{"add_whitelist":{"addresses":["%s"]}}`, uaddr)
	juno.ExecuteContract(ctx, user.KeyName, tfCoreContractAddr, msg)

	// still is one
	res = GetContractConfig(t, ctx, juno, tfCoreContractAddr)
	assert.Assert(t, len(res.Data.AllowedMintAddresses) == 1)

	// add a diff user
	msg = fmt.Sprintf(`{"add_whitelist":{"addresses":["%s"]}}`, uaddr2)
	juno.ExecuteContract(ctx, user.KeyName, tfCoreContractAddr, msg)

	res = GetContractConfig(t, ctx, juno, tfCoreContractAddr)
	assert.Assert(t, len(res.Data.AllowedMintAddresses) == 2)

	// remove user2 from whitelist
	msg = fmt.Sprintf(`{"remove_whitelist":{"addresses":["%s"]}}`, uaddr2)
	juno.ExecuteContract(ctx, user.KeyName, tfCoreContractAddr, msg)

	res = GetContractConfig(t, ctx, juno, tfCoreContractAddr)
	assert.Assert(t, len(res.Data.AllowedMintAddresses) == 1)

	// force transfer 1 token from user to user2
	// '{"force_transfer":{"from":"%s","to":"juno190g5j8aszqhvtg7cprmev8xcxs6csra7xnk3n3","denom":{"denom":"%s","amount":"1"}}}' $KEY_ADDR $FULL_DENOM
	msg = fmt.Sprintf(`{"force_transfer":{"from":"%s","to":"%s","denom":{"denom":"%s","amount":"3"}}}`, uaddr, uaddr2, tfDenom)
	juno.ExecuteContract(ctx, user.KeyName, tfCoreContractAddr, msg)
	CheckBalance(t, ctx, juno, uaddr2, tfDenom, 3)

	msg = fmt.Sprintf(`{"burn_from":{"from":"%s","denom":{"denom":"%s","amount":"1"}}}`, uaddr2, tfDenom)
	juno.ExecuteContract(ctx, user.KeyName, tfCoreContractAddr, msg)
	CheckBalance(t, ctx, juno, uaddr2, tfDenom, 2)

	// mint a token as user2 to user2 addr

	// transfer admin to uaddr2 from contract & remove from being able to mint
	msg = fmt.Sprintf(`{"transfer_admin":{"denom":"%s","new_address":"%s"}}`, tfDenom, uaddr2)
	juno.ExecuteContract(ctx, user.KeyName, tfCoreContractAddr, msg)
	denomAdmin = helpers.GetTokenFactoryAdmin(t, ctx, juno, tfDenom)
	assert.Equal(t, uaddr2, denomAdmin)

	// DENOM WHITELIST
	// adds a denom (Only allow factory/ in the future?)
	msg = fmt.Sprintf(`{"add_denom":{"denoms":["%s"]}}`, "randomdenom")
	juno.ExecuteContract(ctx, user.KeyName, tfCoreContractAddr, msg)

	res = GetContractConfig(t, ctx, juno, tfCoreContractAddr)
	assert.Assert(t, len(res.Data.Denoms) == 1)

	// Remove denom
	msg = fmt.Sprintf(`{"remove_denom":{"denoms":["%s"]}}`, "randomdenom")
	juno.ExecuteContract(ctx, user.KeyName, tfCoreContractAddr, msg)

	res = GetContractConfig(t, ctx, juno, tfCoreContractAddr)
	assert.Assert(t, len(res.Data.Denoms) == 0)

	// Create denom on instantiation
	tfMsg := fmt.Sprintf(`{"allowed_mint_addresses":["%s"],"new_denoms":[{"name":"new","description":"desc","symbol":"crt","decimals":6,"initial_balances":[{"address":"%s","amount":"420"}]}]}`, uaddr, uaddr)
	helpers.InstantiateMsgWithGas(t, ctx, juno, user, tfCoreCodeId, "5000000", "10000000ujuno", tfMsg)
	tfCoreAddr, err := helpers.GetContractAddress(ctx, juno, tfCoreCodeId)
	if err != nil {
		t.Fatal(err)
	}

	t.Log("tfCoreCreateAddr", tfCoreAddr)
	t.Log("err", err)

	tfCreatedDenom := fmt.Sprintf(`factory/%s/crt`, tfCoreAddr)

	res = GetContractConfig(t, ctx, juno, tfCoreAddr)
	assert.Equal(t, len(res.Data.Denoms), 1)
	assert.Equal(t, res.Data.Denoms[0], tfCreatedDenom)

	// Validate admin.
	createdDenomAdmin := helpers.GetTokenFactoryAdmin(t, ctx, juno, tfCreatedDenom)
	assert.Equal(t, tfCoreAddr, createdDenomAdmin)

	// Validate initial balances.
	CheckBalance(t, ctx, juno, uaddr, tfCreatedDenom, 420)
	// do the same thing but through the TF contract query
	createdBalRes := GetCoreContractUserBalance(t, ctx, juno, tfCoreAddr, uaddr, tfCreatedDenom)
	assert.Equal(t, createdBalRes.Data.Amount, "420")
	// Validate supply.
	createdSupply := helpers.GetTokenFactorySupply(t, ctx, juno, tfCreatedDenom)
	assert.Equal(t, createdSupply, "420")

	// !important: debugging
	// t.Log("GetHostRPCAddress", juno.GetHostRPCAddress())
	// testutil.WaitForBlocks(ctx, 20_000, juno)

	// Final Cleanup
	t.Cleanup(func() {
		_ = ic.Close()
	})
}
