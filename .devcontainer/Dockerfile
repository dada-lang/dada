# Use the Debian base image
FROM mcr.microsoft.com/devcontainers/base:bookworm

# Install Node.js
RUN curl -fsSL https://deb.nodesource.com/setup_16.x | bash - \
    && apt-get install -y nodejs

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Add the WASI target
RUN rustup target add wasm32-wasip1

# Install additional tools or dependencies if needed
RUN npm install -g yo generator-code

# Final use
USER vscode

