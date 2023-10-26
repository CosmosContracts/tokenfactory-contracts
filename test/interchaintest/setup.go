package test

import (
	"context"
	"testing"

	// Juno types
	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	feesharetypes "github.com/CosmosContracts/juno/v17/x/feeshare/types"
	tokenfactorytypes "github.com/CosmosContracts/juno/v17/x/tokenfactory/types"

	testutil "github.com/cosmos/cosmos-sdk/types/module/testutil"

	"github.com/docker/docker/client"
	"github.com/strangelove-ventures/interchaintest/v7"
	"github.com/strangelove-ventures/interchaintest/v7/chain/cosmos"
	"github.com/strangelove-ventures/interchaintest/v7/ibc"
	"github.com/strangelove-ventures/interchaintest/v7/testreporter"
	"github.com/stretchr/testify/require"
	"go.uber.org/zap/zaptest"
)

const (
	TF_CORE_FILE = "../../artifacts/juno_tokenfactory_core.wasm"
	MIGRATE_FILE = "../../artifacts/migrate.wasm"
)

var (
	VotingPeriod     = "15s"
	MaxDepositPeriod = "10s"
	Denom            = "ujuno"

	JunoMainRepo = "ghcr.io/cosmoscontracts/juno"

	IBCRelayerImage   = "ghcr.io/cosmos/relayer"
	IBCRelayerVersion = "main"

	JunoVersion = "v17.0.0"

	// SDK v47 Genesis
	defaultGenesisKV = []cosmos.GenesisKV{
		{
			Key:   "app_state.gov.params.voting_period",
			Value: VotingPeriod,
		},
		{
			Key:   "app_state.gov.params.max_deposit_period",
			Value: MaxDepositPeriod,
		},
		{
			Key:   "app_state.gov.params.min_deposit.0.denom",
			Value: Denom,
		},
		// mainnet = 2mil gas, we just require 1 gas for testing
		{
			Key:   "app_state.tokenfactory.params.denom_creation_gas_consume",
			Value: 1,
		},
		{
			Key:   "app_state.tokenfactory.params.denom_creation_fee",
			Value: nil,
		},
	}
)

func junoEncoding() *testutil.TestEncodingConfig {
	cfg := cosmos.DefaultEncoding()

	// register custom juno types
	feesharetypes.RegisterInterfaces(cfg.InterfaceRegistry)
	wasmtypes.RegisterInterfaces(cfg.InterfaceRegistry)
	tokenfactorytypes.RegisterInterfaces(cfg.InterfaceRegistry)

	return &cfg
}

// Basic chain setup for a Juno chain. No relaying
func CreateBaseChain(t *testing.T) []ibc.Chain {
	// Create chain factory with Juno
	numVals := 1
	numFullNodes := 0

	cf := interchaintest.NewBuiltinChainFactory(zaptest.NewLogger(t), []*interchaintest.ChainSpec{
		{
			Name:      "juno",
			Version:   JunoVersion,
			ChainName: "juno1",
			ChainConfig: ibc.ChainConfig{
				GasPrices:      "0ujuno",
				GasAdjustment:  5.0,
				EncodingConfig: junoEncoding(),
				ModifyGenesis:  cosmos.ModifyGenesis(defaultGenesisKV),
			},
			NumValidators: &numVals,
			NumFullNodes:  &numFullNodes,
		},
	})

	// Get chains from the chain factory
	chains, err := cf.Chains(t.Name())
	require.NoError(t, err)

	// juno := chains[0].(*cosmos.CosmosChain)
	return chains
}

func BuildInitialChain(t *testing.T, chains []ibc.Chain) (*interchaintest.Interchain, context.Context, *client.Client, string) {
	// Create a new Interchain object which describes the chains, relayers, and IBC connections we want to use
	ic := interchaintest.NewInterchain()

	for _, chain := range chains {
		ic.AddChain(chain)
	}

	rep := testreporter.NewNopReporter()
	eRep := rep.RelayerExecReporter(t)

	ctx := context.Background()
	client, network := interchaintest.DockerSetup(t)

	err := ic.Build(ctx, eRep, interchaintest.InterchainBuildOptions{
		TestName:         t.Name(),
		Client:           client,
		NetworkID:        network,
		SkipPathCreation: true,
		// This can be used to write to the block database which will index all block data e.g. txs, msgs, events, etc.
		// BlockDatabaseFile: interchaintest.DefaultBlockDatabaseFilepath(),
	})
	require.NoError(t, err)

	return ic, ctx, client, network
}
