package test

import (
	"fmt"
	"testing"

	"github.com/strangelove-ventures/interchaintest/v4"
	"github.com/strangelove-ventures/interchaintest/v4/chain/cosmos"
	"gotest.tools/assert"

	helpers "github.com/CosmosContracts/tokenfactory-contracts/helpers"
)

func TestCw20ConversionMigrateContract(t *testing.T) {
	t.Parallel()

	// Create chain factory with Juno
	chains := CreateBaseChain(t)
	ic, ctx, _, _ := BuildInitialChain(t, chains)
	juno := chains[0].(*cosmos.CosmosChain)

	// User Setup
	users := interchaintest.GetAndFundTestUsers(t, ctx, "default", int64(10_000_000), juno, juno)
	user := users[0]
	uaddr := user.Bech32Address(juno.Config().Bech32Prefix)
	// user2 := users[1]
	// uaddr2 := user2.Bech32Address(CHAIN_PREFIX)

	// Create token-factory denom
	tfDenom := helpers.CreateTokenFactoryDenom(t, ctx, juno, user, "testdenom")
	denomAdmin := helpers.GetTokenFactoryAdmin(t, ctx, juno, tfDenom)
	assert.Equal(t, uaddr, denomAdmin)

	// cw20 (We give outselfs 100 to test in the future)
	cw20Msg := fmt.Sprintf(`{"name":"test","symbol":"aaaa","decimals":6,"initial_balances":[{"address":"%s","amount":"100"}]}`, uaddr)
	_, cw20ContractAddr := helpers.SetupContract(t, ctx, juno, user.KeyName, "../../base_artifacts/cw20_base.wasm", cw20Msg)

	// Tokenfactory Core minter
	tfCoreMsg := fmt.Sprintf(`{"allowed_mint_addresses":[],"existing_denoms":["%s"]}`, tfDenom)
	_, tfCoreContractAddr := helpers.SetupContract(t, ctx, juno, user.KeyName, "../../artifacts/juno_tokenfactory_core.wasm", tfCoreMsg)

	// transfer admin to the contract
	helpers.TransferTokenFactoryAdmin(t, ctx, juno, user, tfCoreContractAddr, tfDenom)
	denomAdmin = helpers.GetTokenFactoryAdmin(t, ctx, juno, tfDenom)
	assert.Equal(t, tfCoreContractAddr, denomAdmin)

	// conversion migrate contract (1 CW20 -> contract -> burn CW20 and mint 1 tf denom)
	migrateCW20Msg := fmt.Sprintf(`{"cw20_token_address":"%s","contract_minter_address":"%s","tf_denom":"%s"}`, cw20ContractAddr, tfCoreContractAddr, tfDenom)
	_, cw20MigrateContractAddr := helpers.SetupContract(t, ctx, juno, user.KeyName, "../../artifacts/migrate.wasm", migrateCW20Msg)

	// Allow the Migration contract to mint through the Tokenfactory Core contract
	msg := fmt.Sprintf(`{"add_whitelist":{"addresses":["%s"]}}`, cw20MigrateContractAddr)
	juno.ExecuteContract(ctx, user.KeyName, tfCoreContractAddr, msg)

	// Ensure the contract config data is set correctly.
	res := GetContractConfig(t, ctx, juno, tfCoreContractAddr)
	assert.Equal(t, res.Data.AllowedMintAddresses[0], cw20MigrateContractAddr)
	assert.Equal(t, res.Data.Denoms[0], tfDenom)

	// actual CW20 testing on the contract
	// ensure user has 0 tf denom balance
	CheckBalance(t, ctx, juno, uaddr, tfDenom, 0)

	// send the message through CW20 -> migrate conversion contract.
	// msg = fmt.Sprintf(`{"send":{"contract":"%s","amount":"%s","msg":"%s"}}`, cw20MigrateContractAddr, "5", b64.StdEncoding.EncodeToString([]byte(`{"receive":{}}`)))
	// txHash, _ := juno.ExecuteContract(ctx, user.KeyName, cw20ContractAddr, msg)
	helpers.CW20Message(t, ctx, juno, user, cw20ContractAddr, cw20MigrateContractAddr, "5", `{"receive":{}}`)

	// we should now have 5 balance of the tf denom
	CheckBalance(t, ctx, juno, uaddr, tfDenom, 5)
	// the cw20 migrate contract should still have 0 balance of this denom (to ensure it does not double mint)
	CheckBalance(t, ctx, juno, cw20MigrateContractAddr, tfDenom, 0)

	// !important: debugging
	// t.Log("GetHostRPCAddress", juno.GetHostRPCAddress())
	// testutil.WaitForBlocks(ctx, 20_000, juno)

	// Final Cleanup
	t.Cleanup(func() {
		_ = ic.Close()
	})
}
