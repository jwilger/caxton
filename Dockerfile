# Multi-stage Dockerfile for Caxton
# Optimized for development with minimal production-ready structure

FROM rust:1.84-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    libc6-compat

WORKDIR /usr/src/caxton

# Copy dependency files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create dummy source files to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "" > src/lib.rs && \
    mkdir -p src/bin && \
    echo "fn main() {}" > src/bin/caxton-cli.rs

# Build dependencies (cached layer)
RUN cargo build --release && \
    rm -rf src target/release/deps/caxton* target/release/caxton*

# Copy actual source code
COPY src/ src/

# Build the actual binaries
RUN cargo build --release

# Runtime stage
FROM alpine:3.20

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    tzdata \
    libgcc

# Create non-root user for security
RUN addgroup -g 1001 -S caxton && \
    adduser -u 1001 -S caxton -G caxton

# Create directories
RUN mkdir -p /var/lib/caxton && \
    mkdir -p /etc/caxton && \
    chown -R caxton:caxton /var/lib/caxton /etc/caxton

# Copy binaries from builder
COPY --from=builder /usr/src/caxton/target/release/caxton /usr/local/bin/
COPY --from=builder /usr/src/caxton/target/release/caxton-cli /usr/local/bin/

# Switch to non-root user
USER caxton

# Set working directory
WORKDIR /var/lib/caxton

# Health check for server process
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD /usr/local/bin/caxton-cli health || exit 1

# Expose ports
EXPOSE 8080 9090

# Default command runs the server
CMD ["/usr/local/bin/caxton", "server", "start"]
