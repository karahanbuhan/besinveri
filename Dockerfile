FROM rust:1.85-alpine

# OpenSSL, kütüphanelerin çalışması için gerekmektedir.
RUN apk add openssl-dev musl-dev

WORKDIR /app
COPY . .

RUN cargo build --release
CMD ["./target/release/besinveri"]

EXPOSE 8099/TCP