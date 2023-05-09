package test

import (
	"fmt"
	"testing"

	"github.com/strangelove-ventures/interchaintest/v4"
	"github.com/strangelove-ventures/interchaintest/v4/chain/cosmos"
	"gotest.tools/assert"

	helpers "github.com/CosmosContracts/tokenfactory-contracts/helpers"
)

func TestNativeConversionMigrateContract(t *testing.T) {
	t.Parallel()

	// Create chain factory with Juno
	chains := CreateBaseChain(t)
	ic, ctx, _, _ := BuildInitialChain(t, chains)

	// Chains
	juno := chains[0].(*cosmos.CosmosChain)
	nativeDenom := juno.Config().Denom

	// User Setup
	users := interchaintest.GetAndFundTestUsers(t, ctx, "default", int64(10_000_000), juno, juno)
	user := users[0]
	uaddr := user.Bech32Address(juno.Config().Bech32Prefix)
	t.Log(uaddr)

	user2 := users[1]
	uaddr2 := user2.Bech32Address(juno.Config().Bech32Prefix)

	// ensure user has some ujuno which is not 0
	CheckBalance(t, ctx, juno, uaddr, nativeDenom, 10_000_000)
	CheckBalance(t, ctx, juno, uaddr2, nativeDenom, 10_000_000)

	// Create token-factory denom
	tfDenom := helpers.CreateTokenFactoryDenom(t, ctx, juno, user, "testdenom")
	assert.Equal(t, uaddr, helpers.GetTokenFactoryAdmin(t, ctx, juno, tfDenom))

	// Tokenfactory Core minter
	tfCoreMsg := fmt.Sprintf(`{"allowed_mint_addresses":[],"denoms":["%s"]}`, tfDenom)
	_, tfCoreContractAddr := helpers.SetupContract(t, ctx, juno, user.KeyName, "../../artifacts/tokenfactory_core.wasm", tfCoreMsg)

	// transfer admin to the contract
	helpers.TransferTokenFactoryAdmin(t, ctx, juno, user, tfCoreContractAddr, tfDenom)
	assert.Equal(t, tfCoreContractAddr, helpers.GetTokenFactoryAdmin(t, ctx, juno, tfDenom))

	// conversion migrate contract (1 native -> 1 tf denom)
	migrateNativeMsg := fmt.Sprintf(`{"burn_denom":"%s","contract_minter_address":"%s","tf_denom":"%s"}`, nativeDenom, tfCoreContractAddr, tfDenom)
	_, naitveMigrateContractAddr := helpers.SetupContract(t, ctx, juno, user.KeyName, "../../artifacts/migrate.wasm", migrateNativeMsg)

	// Allow the Migration contract to mint through the Tokenfactory Core contract
	msg := fmt.Sprintf(`{"add_whitelist":{"addresses":["%s"]}}`, naitveMigrateContractAddr)
	juno.ExecuteContract(ctx, user.KeyName, tfCoreContractAddr, msg)

	// Ensure the contract config data is set correctly.
	res := GetContractConfig(t, ctx, juno, tfCoreContractAddr, uaddr)
	assert.Equal(t, res.Data.AllowedMintAddresses[0], naitveMigrateContractAddr)
	assert.Equal(t, res.Data.Denoms[0], tfDenom)

	// ensure user has 0 tf denom balance
	CheckBalance(t, ctx, juno, uaddr, tfDenom, 0)

	// Execute with an amount
	helpers.ExecuteMsgWithAmount(t, ctx, juno, user2, naitveMigrateContractAddr, fmt.Sprintf("7%s", nativeDenom), `{"convert":{}}`)

	// Ensure we got the correct amount of tokens in exchange for the native token. (no gas prices)
	CheckBalance(t, ctx, juno, uaddr2, tfDenom, 7)
	CheckBalance(t, ctx, juno, uaddr2, nativeDenom, 9_999_993)

	// the migrate contract should still have 0 balance of this denom (to ensure it does not double mint)
	CheckBalance(t, ctx, juno, naitveMigrateContractAddr, tfDenom, 0)

	// // !important: debugging
	// t.Log("GetHostRPCAddress", juno.GetHostRPCAddress())
	// testutil.WaitForBlocks(ctx, 20_000, juno)

	// Final Cleanup
	t.Cleanup(func() {
		_ = ic.Close()
	})
}
