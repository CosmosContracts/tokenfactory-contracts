package test

import (
	"context"
	"testing"

	"github.com/strangelove-ventures/interchaintest/v7/chain/cosmos"
	"github.com/stretchr/testify/require"
	"gotest.tools/assert"
)

func CheckBalance(t *testing.T, ctx context.Context, chain *cosmos.CosmosChain, address, denom string, amount int64) {
	if bal, err := chain.GetBalance(ctx, address, denom); err != nil {
		t.Fatal(err)
	} else {
		t.Log(address, "balance:", bal, denom)
		assert.Equal(t, bal, amount)
	}
}

func GetContractConfig(t *testing.T, ctx context.Context, chain *cosmos.CosmosChain, contract string) GetConfigResponse {
	// tokenfactory_core/src/state.rs
	var cRes GetConfigResponse
	err := chain.QueryContract(ctx, contract, QueryMsg{GetConfig: &struct{}{}}, &cRes)
	require.NoError(t, err)
	t.Log("GetContractConfig", cRes.Data)
	return cRes
}

// TokenFactory Core contract Queries
func GetCoreContractUserBalance(t *testing.T, ctx context.Context, chain *cosmos.CosmosChain, contract, uaddr, tfDenom string) GetBalanceResponse {
	var bRes GetBalanceResponse
	err := chain.QueryContract(ctx, contract, QueryMsg{GetBalance: &GetBalance{Address: uaddr, Denom: tfDenom}}, &bRes)
	require.NoError(t, err)
	return bRes
}

func GetCoreContractUserBalanceAll(t *testing.T, ctx context.Context, chain *cosmos.CosmosChain, contract, uaddr string) GetAllBalancesResponse {
	var bRes GetAllBalancesResponse
	err := chain.QueryContract(ctx, contract, QueryMsg{GetAllBalances: &GetAllBalances{Address: uaddr}}, &bRes)
	require.NoError(t, err)
	return bRes
}
