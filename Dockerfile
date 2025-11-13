FROM rust:latest as builder

WORKDIR /usr/src/chat-app
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /va/lib/apt/lists/*

COPY --from=builder /usr/src/chat-app/target/release/chat-app /usr/local/bin/chat-app

ENTRYPOINT ["chat-app"]
CMD ["client", "--address", "127.0.0.1:8080"]

