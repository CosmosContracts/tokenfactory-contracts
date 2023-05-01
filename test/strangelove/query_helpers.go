package test

// import (
// 	"context"
// 	"strings"
// 	"testing"

// 	"github.com/strangelove-ventures/interchaintest/v4/chain/cosmos"
// 	"github.com/stretchr/testify/require"
// )

// re-arrange these variables to make more sense
// func GetAddressesEntries(t *testing.T, ctx context.Context, chain *cosmos.CosmosChain, contract string, uaddr string) JournalEntriesResponse {
// 	var jer JournalEntriesResponse
// 	err := chain.QueryContract(ctx, contract, QueryMsg{GetEntries: &GetEntries{Address: uaddr}}, &jer)
// 	require.NoError(t, err)
// 	return jer
// }

// func GetWhitelistAddresses(t *testing.T, ctx context.Context, chain *cosmos.CosmosChain, contract string) WhitelistResponse {
// 	var resp WhitelistResponse
// 	err := chain.QueryContract(ctx, contract, QueryMsg{GetWhitelist: &struct{}{}}, &resp)
// 	require.NoError(t, err)
// 	t.Log("\n\nWhitelistResponse-> " + strings.Join(resp.Data, ","))
// 	return resp
// }
