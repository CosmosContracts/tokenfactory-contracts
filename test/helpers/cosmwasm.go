package test

import (
	"context"
	b64 "encoding/base64"
	"encoding/json"
	"fmt"
	"testing"

	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	"github.com/strangelove-ventures/interchaintest/v4/chain/cosmos"
	"github.com/strangelove-ventures/interchaintest/v4/ibc"
	"github.com/strangelove-ventures/interchaintest/v4/testutil"
	"github.com/stretchr/testify/require"
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

func InstantiateMsgWithGas(t *testing.T, ctx context.Context, chain *cosmos.CosmosChain, user *ibc.Wallet, codeId, gas, coinAmt, message string) {
	// TODO: in the future (SDK v47 with genesis params) change this to not use amount :)
	cmd := []string{"junod", "tx", "wasm", "instantiate", codeId, message,
		"--node", chain.GetRPCAddress(),
		"--home", chain.HomeDir(),
		"--chain-id", chain.Config().ChainID,
		"--from", user.KeyName,
		"--gas", gas,
		"--amount", coinAmt,
		"--label", "contract" + codeId,
		"--keyring-dir", chain.HomeDir(),
		"--keyring-backend", keyring.BackendTest,
		"--no-admin",
		"-y",
	}
	stdout, _, err := chain.Exec(ctx, cmd, nil)
	require.NoError(t, err)

	debugOutput(t, string(stdout))

	if err := testutil.WaitForBlocks(ctx, 2, chain); err != nil {
		t.Fatal(err)
	}
}

type QueryContractResponse struct {
	Contracts []string `json:"contracts"`
}

func GetContractAddress(ctx context.Context, chain *cosmos.CosmosChain, codeId string) (string, error) {
	cmd := []string{"junod", "q", "wasm", "list-contract-by-code", codeId,
		"--output", "json",
		"--node", chain.GetRPCAddress(),
		"--home", chain.HomeDir(),
		"--chain-id", chain.Config().ChainID,
	}
	stdout, _, err := chain.Exec(ctx, cmd, nil)
	if err != nil {
		return "", fmt.Errorf("error getting contract address: %w", err)
	}

	contactsRes := QueryContractResponse{}
	if err := json.Unmarshal([]byte(stdout), &contactsRes); err != nil {
		return "", fmt.Errorf("error unmarshalling contract address: %w", err)
	}

	contractAddress := contactsRes.Contracts[len(contactsRes.Contracts)-1]
	return contractAddress, nil
}

func ExecuteMsgWithAmount(t *testing.T, ctx context.Context, chain *cosmos.CosmosChain, user *ibc.Wallet, contractAddr, amount, message string) {
	// amount is #utoken

	// There has to be a way to do this in ictest?
	cmd := []string{"junod", "tx", "wasm", "execute", contractAddr, message,
		"--node", chain.GetRPCAddress(),
		"--home", chain.HomeDir(),
		"--chain-id", chain.Config().ChainID,
		"--from", user.KeyName,
		"--gas", "500000",
		"--amount", amount,
		"--keyring-dir", chain.HomeDir(),
		"--keyring-backend", keyring.BackendTest,
		"-y",
	}
	_, _, err := chain.Exec(ctx, cmd, nil)
	require.NoError(t, err)

	// t.Log("msg", cmd)
	// t.Log("ExecuteMsgWithAmount", string(stdout))

	if err := testutil.WaitForBlocks(ctx, 2, chain); err != nil {
		t.Fatal(err)
	}
}

func CW20Message(t *testing.T, ctx context.Context, chain *cosmos.CosmosChain, user *ibc.Wallet, cw20ContractAddr, actionContractAddr, amount, message string) {
	msg := fmt.Sprintf(`{"send":{"contract":"%s","amount":"%s","msg":"%s"}}`, actionContractAddr, amount, b64.StdEncoding.EncodeToString([]byte(message)))

	// not enough gas... used 200k but needs more. So doing manually.
	// txHash, _ := chain.ExecuteContract(ctx, user.KeyName, cw20ContractAddr, msg)

	cmd := []string{"junod", "tx", "wasm", "execute", cw20ContractAddr, msg,
		"--node", chain.GetRPCAddress(),
		"--home", chain.HomeDir(),
		"--chain-id", chain.Config().ChainID,
		"--from", user.KeyName,
		"--gas", "500000",
		"--keyring-dir", chain.HomeDir(),
		"--keyring-backend", keyring.BackendTest,
		"-y",
	}
	_, _, err := chain.Exec(ctx, cmd, nil)
	require.NoError(t, err)

	// print stdout
	// t.Log("CW20Message", string(stdout))

	if err := testutil.WaitForBlocks(ctx, 2, chain); err != nil {
		t.Fatal(err)
	}
}
