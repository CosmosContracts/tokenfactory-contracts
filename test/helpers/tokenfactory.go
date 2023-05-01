package test

import (
	"context"
	"encoding/json"
	"strconv"
	"testing"

	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	"github.com/strangelove-ventures/interchaintest/v4/chain/cosmos"
	"github.com/strangelove-ventures/interchaintest/v4/ibc"
	"github.com/strangelove-ventures/interchaintest/v4/testutil"
	"github.com/stretchr/testify/require"

	tokenfactorytypes "github.com/CosmWasm/token-factory/x/tokenfactory/types"
)

const CHAIN_PREFIX = "juno"

func CreateTokenFactoryDenom(t *testing.T, ctx context.Context, chain *cosmos.CosmosChain, user *ibc.Wallet, subDenomName string) (fullDenom string) {
	cmd := []string{"junod", "tx", "tokenfactory", "create-denom", subDenomName,
		"--node", chain.GetRPCAddress(),
		"--home", chain.HomeDir(),
		"--chain-id", chain.Config().ChainID,
		"--from", user.KeyName,
		"--keyring-dir", chain.HomeDir(),
		"--keyring-backend", keyring.BackendTest,
		"-y",
	}
	_, _, err := chain.Exec(ctx, cmd, nil)
	require.NoError(t, err)

	err = testutil.WaitForBlocks(ctx, 2, chain)
	require.NoError(t, err)

	return "factory/" + user.Bech32Address(CHAIN_PREFIX) + "/" + subDenomName
}

func MintTokenFactoryDenom(t *testing.T, ctx context.Context, chain *cosmos.CosmosChain, admin *ibc.Wallet, toWallet *ibc.Wallet, amount uint64, fullDenom string) {
	denom := strconv.FormatUint(amount, 10) + fullDenom

	// mint new tokens to the account
	cmd := []string{"junod", "tx", "tokenfactory", "mint-to", toWallet.Bech32Address(CHAIN_PREFIX), denom,
		"--node", chain.GetRPCAddress(),
		"--home", chain.HomeDir(),
		"--chain-id", chain.Config().ChainID,
		"--from", admin.KeyName,
		"--keyring-dir", chain.HomeDir(),
		"--keyring-backend", keyring.BackendTest,
		"-y",
	}
	_, _, err := chain.Exec(ctx, cmd, nil)
	require.NoError(t, err)

	err = testutil.WaitForBlocks(ctx, 2, chain)
	require.NoError(t, err)
}
func TransferTokenFactoryAdmin(t *testing.T, ctx context.Context, chain *cosmos.CosmosChain, currentAdmin *ibc.Wallet, newAdmin *ibc.Wallet, fullDenom string) {
	cmd := []string{"junod", "tx", "tokenfactory", "change-admin", fullDenom, newAdmin.Bech32Address(CHAIN_PREFIX),
		"--node", chain.GetRPCAddress(),
		"--home", chain.HomeDir(),
		"--chain-id", chain.Config().ChainID,
		"--from", currentAdmin.KeyName,
		"--keyring-dir", chain.HomeDir(),
		"--keyring-backend", keyring.BackendTest,
		"-y",
	}
	_, _, err := chain.Exec(ctx, cmd, nil)
	require.NoError(t, err)

	err = testutil.WaitForBlocks(ctx, 2, chain)
	require.NoError(t, err)
}

func GetTokenFactoryAdmin(t *testing.T, ctx context.Context, chain *cosmos.CosmosChain, fullDenom string) string {
	// $BINARY q tokenfactory denom-authority-metadata $FULL_DENOM

	// tokenfactorytypes.QueryDenomAuthorityMetadataRequest{
	// 	Denom: fullDenom,
	// }
	cmd := []string{"junod", "query", "tokenfactory", "denom-authority-metadata", fullDenom,
		"--node", chain.GetRPCAddress(),
		"--chain-id", chain.Config().ChainID,
		"--output", "json",
	}
	stdout, _, err := chain.Exec(ctx, cmd, nil)
	require.NoError(t, err)

	results := &tokenfactorytypes.DenomAuthorityMetadata{}
	err = json.Unmarshal(stdout, results)
	require.NoError(t, err)

	err = testutil.WaitForBlocks(ctx, 2, chain)
	require.NoError(t, err)

	// tokenfactorytypes.DenomAuthorityMetadata{
	// 	Admin: ...,
	// }
	return results.Admin
}
