package test

import (
	"context"
	"testing"

	"github.com/strangelove-ventures/interchaintest/v4/chain/cosmos"
)

func SetupContract(t *testing.T, ctx context.Context, chain *cosmos.CosmosChain, keyname string, fileLoc string, message string) (codeId, contract string) {
	codeId, err := chain.StoreContract(ctx, keyname, fileLoc)
	if err != nil {
		t.Fatal(err)
	}
	// require.Equal(t, "1", codeId)

	contractAddr, err := chain.InstantiateContract(ctx, keyname, codeId, message, true)
	if err != nil {
		t.Fatal(err)
	}
	// t.Log(contractAddr)

	return codeId, contractAddr
}

// func CW20Message(t *testing.T, ctx context.Context, chain *cosmos.CosmosChain, user *ibc.Wallet, cw20ContractAddr, actionContractAddr, amount, message string) {
// 	msg := fmt.Sprintf(`{"send":{"contract":"%s","amount":"%s","msg":"%s"}}`, actionContractAddr, amount, b64.StdEncoding.EncodeToString([]byte(message)))
// 	txHash, _ := chain.ExecuteContract(ctx, user.KeyName, cw20ContractAddr, msg)
// }
