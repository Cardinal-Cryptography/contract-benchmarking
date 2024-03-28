define RUN_WITH_CONTEXT
docker run --rm \
	-v $(PWD)/ink:/workspace/ink \
	-v $(PWD)/polkadot-sdk:/workspace/polkadot-sdk \
	-v $(PWD)/flipper:/workspace/flipper \
	-v $(PWD)/dex:/workspace/dex \
	riscv-contract-builder
endef

.PHONY: help
help: ## Displays this help
	@awk 'BEGIN {FS = ":.*##"; printf "Usage:\n  make \033[1;36m<target>\033[0m\n\nTargets:\n"} /^[a-zA-Z0-9_-]+:.*?##/ { printf "  \033[1;36m%-25s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

clean: ## Remove build artifacts
	@rm -rf flipper/artifacts
	@rm -rf dex/artifacts

# ======================================================================================================================
# Docker operations
# ======================================================================================================================

.PHONY: build-image
build-image: ## Build the image
	@docker build --tag riscv-contract-builder -f Dockerfile .

.PHONY: push-image
push-image: ## Push the image
	@docker tag riscv-contract-builder:latest public.ecr.aws/p6e8q1z1/riscv-contract-builder:latest
	@docker push public.ecr.aws/p6e8q1z1/riscv-contract-builder:latest

.PHONY: pull-image
pull-image: ## Pull the image
	@docker pull public.ecr.aws/p6e8q1z1/riscv-contract-builder:latest
	@docker tag public.ecr.aws/p6e8q1z1/riscv-contract-builder:latest riscv-contract-builder:latest

# ======================================================================================================================
# Flipper contract
# ======================================================================================================================

.PHONY: build-flipper-wasm
build-flipper-wasm: ## Build the flipper contract (wasm target)
	@$(RUN_WITH_CONTEXT) build --release --target wasm --manifest-path flipper/Cargo.toml
	@mkdir -p flipper/artifacts && cp flipper/target/ink/flipper.wasm flipper/artifacts

.PHONY: build-flipper-riscv
build-flipper-riscv: ## Build the flipper contract (riscv target)
	@$(RUN_WITH_CONTEXT) build --release --target riscv --manifest-path flipper/Cargo.toml
	@mkdir -p flipper/artifacts && cp flipper/target/ink/flipper.riscv flipper/artifacts

flipper/artifacts/flipper.wasm:
	@$(MAKE) build-flipper-wasm
flipper/artifacts/flipper.riscv:
	@$(MAKE) build-flipper-riscv

.PHONY: run-flipper-simulation
run-flipper-simulation: flipper/artifacts/flipper.wasm flipper/artifacts/flipper.riscv ## Run the flipper contract simulation
	@cd simulation && cargo run --release --bin flipper-simulation

# ======================================================================================================================
# Common DEX
# ======================================================================================================================

.PHONY: build-dex-wasm
build-dex-wasm: ## Build dex contracts (wasm target)
	@$(RUN_WITH_CONTEXT) build --release --target wasm --manifest-path dex/PSP22/Cargo.toml --features contract
	@$(RUN_WITH_CONTEXT) build --release --target wasm --manifest-path dex/wrapped-azero/Cargo.toml
	@$(RUN_WITH_CONTEXT) build --release --target wasm --manifest-path dex/pair/Cargo.toml
	@$(RUN_WITH_CONTEXT) build --release --target wasm --manifest-path dex/router/Cargo.toml
	@$(RUN_WITH_CONTEXT) build --release --target wasm --manifest-path dex/factory/Cargo.toml
	@mkdir -p dex/artifacts && \
		cp dex/PSP22/target/ink/psp22.wasm dex/artifacts && \
		cp dex/wrapped-azero/target/ink/wrapped_azero.wasm dex/artifacts && \
		cp dex/pair/target/ink/pair_contract.wasm dex/artifacts && \
		cp dex/router/target/ink/router_contract.wasm dex/artifacts && \
		cp dex/factory/target/ink/factory_contract.wasm dex/artifacts

.PHONY: build-dex-riscv
build-dex-riscv: ## Build dex contracts (riscv target)
	@$(RUN_WITH_CONTEXT) build --release --target riscv --manifest-path dex/PSP22/Cargo.toml --features contract
	@$(RUN_WITH_CONTEXT) build --release --target riscv --manifest-path dex/wrapped-azero/Cargo.toml
	@$(RUN_WITH_CONTEXT) build --release --target riscv --manifest-path dex/pair/Cargo.toml
	@$(RUN_WITH_CONTEXT) build --release --target riscv --manifest-path dex/router/Cargo.toml
	@$(RUN_WITH_CONTEXT) build --release --target riscv --manifest-path dex/factory/Cargo.toml
	@mkdir -p dex/artifacts && \
		cp dex/PSP22/target/ink/psp22.riscv dex/artifacts && \
		cp dex/wrapped-azero/target/ink/wrapped_azero.riscv dex/artifacts && \
		cp dex/pair/target/ink/pair_contract.riscv dex/artifacts && \
		cp dex/router/target/ink/router_contract.riscv dex/artifacts && \
		cp dex/factory/target/ink/factory_contract.riscv dex/artifacts

dex/artifacts/psp22.wasm:
	@$(MAKE) build-dex-wasm
dex/artifacts/psp22.riscv:
	@$(MAKE) build-dex-riscv
dex/artifacts/wrapped_azero.wasm:
	@$(MAKE) build-dex-wasm
dex/artifacts/wrapped_azero.riscv:
	@$(MAKE) build-dex-riscv
dex/artifacts/pair_contract.wasm:
	@$(MAKE) build-dex-wasm
dex/artifacts/pair_contract.riscv:
	@$(MAKE) build-dex-riscv
dex/artifacts/router_contract.wasm:
	@$(MAKE) build-dex-wasm
dex/artifacts/router_contract.riscv:
	@$(MAKE) build-dex-riscv
dex/artifacts/factory_contract.wasm:
	@$(MAKE) build-dex-wasm
dex/artifacts/factory_contract.riscv:
	@$(MAKE) build-dex-riscv

.PHONY: run-dex-simulation
run-dex-simulation: dex/artifacts/psp22.wasm dex/artifacts/psp22.riscv dex/artifacts/wrapped_azero.wasm dex/artifacts/wrapped_azero.riscv dex/artifacts/pair_contract.wasm dex/artifacts/pair_contract.riscv dex/artifacts/router_contract.wasm dex/artifacts/router_contract.riscv dex/artifacts/factory_contract.wasm dex/artifacts/factory_contract.riscv ## Run the dex contracts simulation
	@cd simulation && cargo run --release --bin dex-simulation
