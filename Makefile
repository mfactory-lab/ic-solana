#!/usr/bin/make

.DEFAULT_GOAL: help

help: ## Show this help
	@printf "\033[33m%s:\033[0m\n" 'Available commands'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[32m%-18s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# ----------------------------------------------------------------------------------------------------------------------

.PHONY: start
start: ## Start the canisters
	RUST_BACKTRACE=1 dfx start --clean

.PHONY: build
build: ## Build the canisters
	./scripts/build

.PHONY: examples
examples: ## Run examples
	./scripts/examples

.PHONY: metrics
metrics: ## Get metrics
	dfx canister call ic-solana-provider getMetrics '()'

.PHONY: test
test: fetch-pocket-ic ## Run tests
	$(eval POCKET_IC_BIN?=$(shell pwd)/pocket-ic)

	## @dfx build test_canister; 
	@export IC_SOLANA_PROVIDER_PATH=./target/wasm32-unknown-unknown/release/ic_solana_provider.wasm.gz; 
	@$(MAKE) build; 
	@ POCKET_IC_BIN=${POCKET_IC_BIN} cargo test $(TEST) --no-fail-fast $(if $(TEST_NAME),-- $(TEST_NAME) --nocapture,-- --nocapture); 

.PHONY: fetch-pocket-ic
fetch-pocket-ic: ## Fetches working pocket-ic binary for tests
	./scripts/fetch-pocket-ic

.PHONY: test-e2e
test-e2e: build ## Run e2e tests
	dfx canister call test_canister test

.PHONY: clean
clean: ## Cleanup
	rm -rf .dfx
	rm -rf node_modules
	rm -rf src/declarations
	cargo clean

%::
	@true
