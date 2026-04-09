# Build stage
FROM rust:1.88-slim AS builder

WORKDIR /usr/src/dump1090_rs

# Install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libsoapysdr-dev \
    libclang-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace library first (better caching)
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY benches/ benches/

# Copy the dump1090_rs binary crate
COPY dump1090_rs/ dump1090_rs/

# Build the project in release mode
RUN RUSTFLAGS="-C target-cpu=native" cargo build --release --workspace && \
    cp target/release/dump1090_rs /usr/local/bin/

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libsoapysdr0.8 \
    soapysdr-module-all \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/bin/dump1090_rs /usr/local/bin/

# Expose the default port
EXPOSE 30002

# Default command
ENTRYPOINT ["dump1090_rs"]
CMD ["--host", "0.0.0.0", "--port", "30002"]
