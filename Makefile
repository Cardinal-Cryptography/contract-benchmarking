define RUN_WITH_CONTEXT
docker run --rm \
	-v $(PWD)/ink:/workspace/ink \
	-v $(PWD)/polkadot-sdk:/workspace/polkadot-sdk \
	-v $(PWD)/flipper:/workspace/flipper \
	contract-builder
endef

.PHONY: build-image
build-image: ## Build the image
	@docker build --tag contract-builder -f Dockerfile .

.PHONY: build-flipper-wasm
build-flipper-wasm: ## Build the flipper contract (wasm target)
	@$(RUN_WITH_CONTEXT) build --release --target wasm --manifest-path flipper/Cargo.toml --offline
	@mkdir -p flipper/artifacts && cp flipper/target/ink/flipper.wasm flipper/artifacts

.PHONY: build-flipper-riscv
build-flipper-riscv: ## Build the flipper contract (riscv target)
	@$(RUN_WITH_CONTEXT) build --release --target riscv --manifest-path flipper/Cargo.toml
	@mkdir -p flipper/artifacts && cp flipper/target/ink/flipper.riscv flipper/artifacts

help: ## Displays this help
	@awk 'BEGIN {FS = ":.*##"; printf "Usage:\n  make \033[1;36m<target>\033[0m\n\nTargets:\n"} /^[a-zA-Z0-9_-]+:.*?##/ { printf "  \033[1;36m%-25s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)
