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
	return cRes
}
