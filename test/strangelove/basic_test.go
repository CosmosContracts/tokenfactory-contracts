package test

import (
	"testing"

	"github.com/strangelove-ventures/interchaintest/v4"
	"github.com/strangelove-ventures/interchaintest/v4/chain/cosmos"

	// "github.com/stretchr/testify/require"

	helpers "github.com/CosmosContracts/tokenfactory-contracts/helpers"
)

const CHAIN_PREFIX = "juno"

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
	// keyname := user.KeyName
	// uaddr := user.Bech32Address(CHAIN_PREFIX)

	user2 := users[1]
	// uaddr2 := user2.Bech32Address(CHAIN_PREFIX)

	// NEW CONTRACTS HERE:
	// cw20Msg := fmt.Sprintf(`{"name":"test","symbol":"aaaa","decimals":6,"initial_balances":[{"address":"%s","amount":"100"}]}`, uaddr)
	// cw20ContractAddr := helpers.SetupContract(t, ctx, juno, keyname, "../base_artifacts/cw20_base.wasm", cw20Msg)

	// tfCoreMsg := fmt.Sprintf(`{"allowed_mint_addresses":[],"denoms":["%s"]}`, denom)
	// tfCoreContractAddr := SetupContract(t, ctx, juno, keyname, "../artifacts/tokenfactory_core.wasm", tfCoreMsg)

	denomName := helpers.CreateTokenFactoryDenom(t, ctx, juno, user, "testdenom")

	// get admin
	// denomAdmin := helpers.GetTokenFactoryAdmin(t, ctx, juno, denomName)
	// t.Log("denomAdmin", denomAdmin)

	// balance, _ := juno.GetBalance(ctx, user2.Bech32Address(CHAIN_PREFIX), denomName)
	helpers.MintTokenFactoryDenom(t, ctx, juno, user, user2, 100, denomName)
	// newBalance, _ := juno.GetBalance(ctx, user2.Bech32Address(CHAIN_PREFIX), denomName)

	// old
	// old
	// old
	// old
	// Contract Testing
	// cw20_codeId, err := juno.StoreContract(ctx, keyname, "../artifacts/journaling.wasm")

	// // Execute on the chain and add an entry for a user
	// msg := fmt.Sprintf(`{"submit":{"entries":[{"date":"%s","title":"%s","repo_pr":"%s","notes":"%s"}]}}`, "Apr-26-2023", "My title here", "https://reece.sh", "note")
	// _, err = juno.ExecuteContract(ctx, keyname, contract, msg)
	// if err != nil {
	// 	t.Fatal(err)
	// }

	// res := GetAddressesEntries(t, ctx, juno, contract, uaddr)
	// for k, v := range res.Data {
	// 	t.Log(k, v)
	// }

	// // == submit another entry ==
	// msg = fmt.Sprintf(`{"submit":{"entries":[{"date":"%s","title":"%s","repo_pr":"%s","notes":"%s"}]}}`, "Apr-26-2023", "2nd title", "github.com/2", "")
	// _, err = juno.ExecuteContract(ctx, keyname, contract, msg)
	// if err != nil {
	// 	t.Fatal(err)
	// }
	// res = GetAddressesEntries(t, ctx, juno, contract, uaddr)
	// for k, v := range res.Data {
	// 	t.Log(k, v)
	// }

	// Final Cleanup
	t.Cleanup(func() {
		_ = ic.Close()
	})
}
