FROM debian:13

# Install required packages including rustup from apt
RUN apt-get update && apt-get install -y \
    rustup \
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

# Initialize rustup with the stable toolchain
RUN rustup default stable

# Add cargo to PATH
ENV PATH="/home/dev/.cargo/bin:${PATH}"
