package test

// TODO: Auto generate in the future from Rust types -> Go types?
// Execute types are not needed here. We just use strings. Could add though in the future and to_string it

// EntryPoint
type QueryMsg struct {
	GetEntries   *GetEntries `json:"get_entries,omitempty"`
	GetWhitelist *struct{}   `json:"get_whitelist,omitempty"`
}

// Response Types (json is always 'data' from the chain return value)
type JournalEntriesResponse struct {
	Data map[string]JournalEntry `json:"data"`
}

type WhitelistResponse struct {
	Data []string `json:"data"`
}

// Middleware
type GetEntries struct {
	Address string `json:"address"`
}

// Base Data Types
type JournalEntry struct {
	Date   string `json:"date"`
	Title  string `json:"title"`
	RepoPr string `json:"repo_pr"`
	Notes  string `json:"notes"`
}
