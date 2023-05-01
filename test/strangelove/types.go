package test

// TODO: Auto generate in the future from Rust types -> Go types?
// Execute types are not needed here. We just use strings. Could add though in the future and to_string it

// EntryPoint
type QueryMsg struct {
	// GetEntries   *GetEntries `json:"get_entries,omitempty"`
	GetConfig *struct{} `json:"get_config,omitempty"`
}

// Response Types (json is always 'data' from the chain return value)
type GetConfigResponse struct {
	Data *ConfigTfCore `json:"data"`
}

// type WhitelistResponse struct {
// 	Data []string `json:"data"`
// }

// // Middleware
// type GetEntries struct {
// 	Address string `json:"address"`
// }

// Base Data Types
type ConfigTfCore struct {
	Manager              string   `json:"manager"`
	AllowedMintAddresses []string `json:"allowed_mint_addresses"`
	Denoms               []string `json:"denoms"`
}
