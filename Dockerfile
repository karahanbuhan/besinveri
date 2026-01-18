FROM rust:1.85-alpine

# OpenSSL, kütüphanelerin çalışması için gerekmektedir.
RUN apk add --no-cache musl-dev

WORKDIR /app
COPY . .

RUN cargo build --release
CMD ["./target/release/besinveri-api"]

EXPOSE 8099/TCP