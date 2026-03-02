FROM mcr.microsoft.com/devcontainers/base:ubuntu

# Install Python 3 and pip
RUN apt-get update && apt-get install -y python3 python3-pip python3-venv && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain as vscode user
USER vscode
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && . "$HOME/.cargo/env" \
    && rustup component add rust-analyzer
ENV PATH="/home/vscode/.cargo/bin:${PATH}"
