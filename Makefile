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

.PHONY: e2e
e2e: build ## Run e2e tests
	./scripts/e2e

.PHONY: test
test: ## Run tests
#	cargo build --release --target wasm32-unknown-unknown --package ic-solana-provider
#	cargo test -- --nocapture
	dfx build test_canister; \
	@{ \
	  make -f ./src/test_canister/Makefile build; \
		export IC_SOLANA_PROVIDER_PATH=./target/wasm32-unknown-unknown/release/ic_solana_provider.wasm.gz; \
		export SCHNORR_CANISTER_PATH=./target/wasm32-unknown-unknown/release/test_canister.wasm.gz; \
		$(MAKE) build; \
		cargo test --test integration_tests $(if $(TEST_NAME),-- $(TEST_NAME) --nocapture,-- --nocapture); \
	}

.PHONY: clean
clean: ## Cleanup
	rm -rf .dfx
	rm -rf node_modules
	rm -rf src/declarations
	cargo clean

%::
	@true
