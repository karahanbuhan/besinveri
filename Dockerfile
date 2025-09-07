FROM rust:1.85-slim

WORKDIR /app
COPY . .

RUN cargo build --release
CMD ["./target/release/besinveri"]