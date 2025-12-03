# Builds the chip-in-inventory application using a multi-stage build for a minimal final image.
#
# Stage 1 (builder): Compiles the Rust application with its build dependencies.
# Stage 2 (final):   Copies the compiled binary and web assets into a slim Debian image.

# --- Stage 1: Build the application ---
FROM rust:1.91 AS builder

# Install build tools needed for some Rust crate dependencies that compile C code.
RUN apt-get update && apt-get install -y protobuf-compiler  && rm -rf /var/lib/apt/lists/*


# Install dependencies needed for building
WORKDIR /usr/src/chip-in-inventory

# Copy the source code
COPY . .

# Build the application in release mode
RUN cargo build --release

# --- Stage 2: Create the final, smaller image ---
FROM debian:bookworm-slim

# Copy the built binary from the builder stage
COPY --from=builder /usr/src/chip-in-inventory/target/release/chip-in-inventory /usr/local/bin/chip-in-inventory
# Copy the web UI files
COPY webroot ./webroot

# Set the command to run the application
CMD ["chip-in-inventory"]
