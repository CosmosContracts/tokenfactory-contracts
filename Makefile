BASENAME = $(shell basename $(shell pwd))

.PHONY: compile
compile:
	docker run --rm -v "$(shell pwd)":/code --mount type=volume,source="$(BASENAME)_cache",target=/code/target --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry cosmwasm/workspace-optimizer:0.12.10

.PHONY: clippy
clippy:
	cargo clippy
	cargo fmt

.PHONY: test
test:	
	cargo test -- --nocapture

.PHONY: ictest-basic
ictest-basic:
	cd test/interchaintest && go test -race -v -run TestBasicContract .

.PHONY: ictest-conversion-cw20
ictest-conversion-cw20:
	cd test/interchaintest && go test -race -v -run TestCw20ConversionMigrateContract .

.PHONY: ictest-conversion-native
ictest-conversion-native:
	cd test/interchaintest && go test -race -v -run TestNativeConversionMigrateContract .

