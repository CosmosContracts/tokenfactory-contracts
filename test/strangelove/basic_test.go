package test

import (
	"fmt"
	"testing"

	"github.com/strangelove-ventures/interchaintest/v4"
	"github.com/strangelove-ventures/interchaintest/v4/chain/cosmos"

	helpers "github.com/CosmosContracts/tokenfactory-contracts/helpers"
)

const CHAIN_PREFIX = "juno"

// TODO: Can we change timeout_commit to 1-2 seconds?
func TestBasicContract(t *testing.T) {
	t.Parallel()

	// Create chain factory with Juno
	chains := CreateBaseChain(t)
	juno := chains[0].(*cosmos.CosmosChain)

	// Builds the chain for testing
	ic, ctx, _, _ := BuildInitialChain(t, chains)

	// User Setup
	users := interchaintest.GetAndFundTestUsers(t, ctx, "default", int64(10_000_000), juno, juno)
	user := users[0]
	uaddr := user.Bech32Address(CHAIN_PREFIX)

	user2 := users[1]
	// uaddr2 := user2.Bech32Address(CHAIN_PREFIX)

	// NEW CONTRACTS HERE:
	cw20Msg := fmt.Sprintf(`{"name":"test","symbol":"aaaa","decimals":6,"initial_balances":[{"address":"%s","amount":"100"}]}`, uaddr)
	cw20ContractAddr := helpers.SetupContract(t, ctx, juno, user.KeyName, "../../base_artifacts/cw20_base.wasm", cw20Msg)

	tfDenom := helpers.CreateTokenFactoryDenom(t, ctx, juno, user, "testdenom")

	// TODO get admin -does not work yet
	// denomAdmin := helpers.GetTokenFactoryAdmin(t, ctx, juno, denomName)
	// t.Log("denomAdmin", denomAdmin)

	// balance, _ := juno.GetBalance(ctx, user2.Bech32Address(CHAIN_PREFIX), denomName)
	helpers.MintTokenFactoryDenom(t, ctx, juno, user, user2, 100, tfDenom)
	// newBalance, _ := juno.GetBalance(ctx, user2.Bech32Address(CHAIN_PREFIX), denomName)

	// Setup Tokenfactory Core contract (mints on your/daos behalf)
	tfCoreMsg := fmt.Sprintf(`{"allowed_mint_addresses":[],"denoms":["%s"]}`, tfDenom)
	tfCoreContractAddr := helpers.SetupContract(t, ctx, juno, user.KeyName, "../../artifacts/tokenfactory_core.wasm", tfCoreMsg)
	t.Log("tfCoreContractAddr", tfCoreContractAddr)

	// Setup the migration contract (convert a cw20 to a native denom (ex: ibc, native, or factory))
	migrateCW20Msg := fmt.Sprintf(`{"cw20_token_address":"%s","contract_minter_address":"%s","tf_denom":"%s"}`, cw20ContractAddr, tfCoreContractAddr, tfDenom)
	cw20MigrateContractAddr := helpers.SetupContract(t, ctx, juno, user.KeyName, "../../artifacts/migrate.wasm", migrateCW20Msg)

	// TODO: native migrate (ujuno -> tf denom. from IBC, native, or factory conversions to TF denom)

	// Allow the Migration contract to mint through the Tokenfactory Core contract
	msg := fmt.Sprintf(`{"add_whitelist":{"addresses":["%s"]}}`, cw20MigrateContractAddr)
	if _, err := juno.ExecuteContract(ctx, user.KeyName, tfCoreContractAddr, msg); err != nil {
		t.Fatal(err)
	}

	// Ensure the data is set correctly.
	// v.Data &{juno1kwxerlhy9qsd48gnd4h2dgdk9kalas5xht8fwe [juno17p9rzwnnfxcjp32un9ug7yhhzgtkhvl9jfksztgw5uh69wac2pgszu8fr9] [factory/juno1kwxerlhy9qsd48gnd4h2dgdk9kalas5xht8fwe/testdenom]}
	v := GetContractConfig(t, ctx, juno, tfCoreContractAddr, uaddr)
	t.Log("v.Data", v.Data)

	// test cw20 (e2e test_cw20_contract)
	// - check tf bal
	// - send 5 cw20 to the migrate contract
	// - Check the balance is now 5 TF
	// ensure the migrate and middleware contract does not have any extra minted (this should never happen)

	// TODO Native
	// do the same, but native. How to do a wasm execute in ictest with amount added?

	// Final Cleanup
	t.Cleanup(func() {
		_ = ic.Close()
	})
}
