FROM rust:1.85-alpine AS builder

# OpenSSL, kütüphanelerin çalışması için gerekmektedir.
RUN apk add --no-cache openssl-dev musl-dev

WORKDIR /app
COPY . .

RUN cargo build --release

# Production stage - use minimal base image
FROM alpine:latest

# Install runtime dependencies only
RUN apk add --no-cache libgcc openssl ca-certificates && \
    addgroup -g 1000 besinveri && \
    adduser -D -u 1000 -G besinveri besinveri

WORKDIR /app

# Copy only the binary from builder
COPY --from=builder /app/target/release/besinveri /app/besinveri
COPY --from=builder /app/migrations /app/migrations
COPY --from=builder /app/db /app/db
COPY --from=builder /app/config.toml /app/config.toml

# Set ownership
RUN chown -R besinveri:besinveri /app

# Switch to non-root user
USER besinveri

CMD ["./besinveri"]

EXPOSE 8099/TCP