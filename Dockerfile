# Use a Rust base image for building the application
FROM rust:latest as builder

# Create a new empty shell project
RUN cargo new --bin bunnylol
WORKDIR /app

# Copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./Rocket.toml ./Rocket.toml
COPY ./src ./src
COPY ./config.yaml ./config.yaml
COPY ./templates ./templates

# Build your application
#RUN rm ./target/release/deps/myapp*
RUN cargo build --release

# Use a minimal runtime image to reduce size
FROM debian:bookworm-slim
COPY --from=builder /app/target/release/bunnylol /app/bunnylol
COPY --from=builder /app/config.yaml /app/config.yaml
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/Rocket.toml /app/Rocket.toml

# Set the command to run your binary
EXPOSE 8000
WORKDIR /app
CMD ["./bunnylol"]
