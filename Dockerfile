FROM ubuntu:20.04

# Install ubuntu packages
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y curl wget libssl-dev gcc build-essential pkg-config git build-essential zstd

# Install rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Install toolchain
RUN wget https://github.com/paritytech/rustc-rv32e-toolchain/releases/download/v1.0.0/rust-rve-nightly-2023-04-05-x86_64-unknown-linux-gnu.tar.zst && \
    tar --zstd -xf rust-rve-nightly-2023-04-05-x86_64-unknown-linux-gnu.tar.zst && \
    mv rve-nightly ~/.rustup/toolchains/ && \
    cp -r ~/.rustup/toolchains/rve-nightly ~/.rustup/toolchains/rv32e-nightly-2023-04-05

# Build cargo-contract
RUN . $HOME/.cargo/env && \
    cargo install --git https://github.com/paritytech/cargo-contract --branch at/riscv

# Install rust component
RUN . $HOME/.cargo/env && \
    rustup component add rust-src --toolchain stable-x86_64-unknown-linux-gnu

# Prepare the workspace
WORKDIR /workspace

# Set the entrypoint to cargo-contract
ENTRYPOINT ["/root/.cargo/bin/cargo", "contract"]
