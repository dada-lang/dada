# Use the Debian base image
FROM mcr.microsoft.com/devcontainers/base:bookworm

# Install Node.js
RUN curl -fsSL https://deb.nodesource.com/setup_16.x | bash - \
    && apt-get install -y nodejs

# Install additional tools or dependencies if needed
RUN npm install -g yo generator-code

# Install OpenSSH client
RUN apt-get update && apt-get install -y openssh-client

USER vscode

# Install Rust and the wasm32-wasip1-threads target
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/home/vscode/.cargo/bin:${PATH}"
RUN rustup install nightly
RUN rustup target add wasm32-wasip1-threads


