FROM rust:nightly

# Install system dependencies
RUN apt-get update && apt-get install -y \
    qemu-system-x86 \
    ovmf \
    llvm \
    clang \
    lld \
    curl \
    git \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install rust components
RUN rustup component add llvm-tools-preview rust-src

# Install just (task runner)
RUN curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to /usr/local/bin

WORKDIR /app
# We don't copy sources here to allow volume mounting during dev
# COPY . .

CMD ["/bin/bash"]
