#!/usr/bin/make

POCKET_IC_BIN := $(shell pwd)/pocket-ic

.DEFAULT_GOAL: help

.PHONY: help
help: ## Show this help
	@printf "\033[33m%s:\033[0m\n" 'Available commands'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[32m%-18s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# ----------------------------------------------------------------------------------------------------------------------

.PHONY: bootstrap
bootstrap: ## Bootstrap
	./scripts/bootstrap

.PHONY: start
start: ## Start local replica
	@RUST_BACKTRACE=1 dfx start --clean

.PHONY: build
build: build.rpc build.wallet  ## Build all canisters

.PHONY: build.rpc
build.rpc: ## Build RPC canister
	./scripts/build --rpc

.PHONY: build.wallet
build.wallet: ## Build wallet canister
	./scripts/build --wallet

.PHONY: did
did: ## Generate did
	./scripts/did $(filter-out $@,$(MAKECMDGOALS))

.PHONY: examples
examples: ## Run examples
	@./scripts/examples

.PHONY: metrics
metrics: ## Fetch metrics
	@dfx canister call ic-solana-rpc getMetrics '()'

.PHONY: test
test: build ## Run tests
	@echo "Running tests..."
	@if [! -f "$(POCKET_IC_BIN)"]; then \
		echo "Pocket IC binary not found. Fetching..."; \
		$(MAKE) fetch-pocket-ic; \
	fi
	cargo test $(TEST) --no-fail-fast -- $(if $(TEST_NAME),$(TEST_NAME),) --nocapture

.PHONY: test-e2e
test-e2e: build ## Run end-to-end tests
	@echo "Running end-to-end tests..."
	dfx canister call e2e test

.PHONY: fetch-pocket-ic
fetch-pocket-ic: ## Fetch the pocket-ic binary for tests if not already present
	./scripts/fetch-pocket-ic

.PHONY: clean
clean: ## Remove build artifacts and dependencies
	rm -rf .dfx node_modules src/declarations
	cargo clean

# ----------------------------------------------------------------------------------------------------------------------

%:
	@:
