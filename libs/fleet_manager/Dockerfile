# Build stage
FROM rust:1.80 as builder

WORKDIR /usr/src/app
# Copy the source code
COPY . .

# Build the release binary
RUN cargo build --release

# Final runtime image
FROM debian:bookworm-slim

# Install necessary root certificates (in case we need to make outbound HTTPS calls later)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/fleet_manager /app/fleet_manager

# Expose the web port
EXPOSE 3000

# Set a default secret token (should be overridden by docker-compose)
ENV FLEET_SECRET_TOKEN=my_secret

# Start the fleet manager
CMD ["/app/fleet_manager"]
