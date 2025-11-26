FROM debian:13

# Install required packages
RUN apt-get update && apt-get install -y \
    bash \
    curl \
    wget \
    psmisc \
    ca-certificates \
    lsb-release \
    openssh-client \
    socat \
    gcc \
    sqlite3 \
    openssl \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create unprivileged user
RUN useradd -m -s /bin/bash dev

# Switch to unprivileged user
USER dev
WORKDIR /home/dev

# Install rustup as the dev user
# Using official rustup installer from https://rustup.rs
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable

# Add cargo to PATH
ENV PATH="/home/dev/.cargo/bin:${PATH}"
