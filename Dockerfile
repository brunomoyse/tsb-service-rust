# Define the build stage
FROM rust:latest as builder

# Create a new empty shell project
RUN USER=root cargo new --bin app
WORKDIR /app

# Copy your source files
COPY ./src ./src
COPY Cargo.toml Cargo.lock ./

# Build your application
RUN cargo build --release

# Define the final stage
FROM debian:stable-slim

# Install needed libraries
RUN apt-get update && apt-get install -y \
    libpq-dev \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Add a non-root user
RUN useradd -ms /bin/bash appuser

# Copy the build artifact from the build stage
COPY --from=builder /app/target/release/tsb /usr/local/bin/

# Change ownership of the binary to the non-root user
RUN chown appuser:appuser /usr/local/bin/tsb

# Set the working directory
WORKDIR /usr/local/bin

# Switch to the non-root user
USER appuser

# Command to run the executable
CMD ["./tsb"]
