# Build stage
FROM rust:1.75 as builder

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY shared/ ./shared/
COPY services/api-gateway/ ./services/api-gateway/

# Build the application
RUN cargo build --release --bin elementa-api-gateway

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false elementa

# Copy binary
COPY --from=builder /app/target/release/elementa-api-gateway /usr/local/bin/

# Copy configuration
COPY config/ /app/config/

# Set ownership
RUN chown -R elementa:elementa /app

USER elementa
WORKDIR /app

EXPOSE 8080 9090

CMD ["elementa-api-gateway"]