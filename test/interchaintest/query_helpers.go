package test

import (
	"context"
	"testing"

	"github.com/strangelove-ventures/interchaintest/v4/chain/cosmos"
	"github.com/stretchr/testify/require"
)

func GetContractConfig(t *testing.T, ctx context.Context, chain *cosmos.CosmosChain, contract string, uaddr string) GetConfigResponse {
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
