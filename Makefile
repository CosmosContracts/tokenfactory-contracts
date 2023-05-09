BASENAME = $(shell basename $(shell pwd))

compile:
	docker run --rm -v "$(shell pwd)":/code --mount type=volume,source="$(BASENAME)_cache",target=/code/target --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry cosmwasm/workspace-optimizer:0.12.10

clippy:
	cargo clippy
	cargo fmt

test:	
	cargo test -- --nocapture

ictest-basic:
	cd test/interchaintest && go test -race -v -run TestBasicContract .


ictest-conversion:
	cd test/interchaintest && go test -race -v -run TestCw20ConversionMigrateContract .

