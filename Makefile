build-image: ## Build the image
	docker build --tag contract-builder -f Dockerfile .

build-flipper-wasm: ## Build the flipper contract (wasm target)
	docker run --rm \
		-v $(PWD)/ink:/workspace/ink \
		-v $(PWD)/polkadot-sdk:/workspace/polkadot-sdk \
		-v $(PWD)/flipper:/workspace/flipper \
		contract-builder build --release --target wasm --manifest-path flipper/Cargo.toml

help: ## Displays this help
	@awk 'BEGIN {FS = ":.*##"; printf "Usage:\n  make \033[1;36m<target>\033[0m\n\nTargets:\n"} /^[a-zA-Z0-9_-]+:.*?##/ { printf "  \033[1;36m%-25s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)