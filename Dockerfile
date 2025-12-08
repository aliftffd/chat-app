FROM rust:latest as builder

WORKDIR /usr/src/chat-app
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

# Install runtime dependencies including SQLite
RUN apt-get update && \
    apt-get install -y \
    ca-certificates \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

# Create directory for database storage
RUN mkdir -p /data

COPY --from=builder /usr/src/chat-app/target/release/chat-app /usr/local/bin/chat-app

# Set environment variable for data directory
ENV XDG_DATA_HOME=/data

# Volume for persistent storage
VOLUME ["/data"]

ENTRYPOINT ["chat-app"]
CMD ["server", "--address", "0.0.0.0:8080"]

