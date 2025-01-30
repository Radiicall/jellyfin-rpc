FROM debian:bookworm

RUN apt-get update && apt-get install -y \
    build-essential \
    ca-certificates \
    curl \
    wget \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:$PATH"

WORKDIR /app

COPY . .

RUN cargo build --release

CMD ["./target/release/jellyfin-rpc"]
