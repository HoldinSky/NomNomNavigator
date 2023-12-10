#   syntax=docker/dockerfile:1
# run in the root of project next commands:
#
# docker build -t nnn-rust-image .
# docker run -d --name <container_name> -p 8080:8080 \
#     -e PG_DATABASE_URL=postgres://[USER]:[PASS]@[HOST]:[PORT]/[DB_NAME] \
#     -e REDIS_DATABASE_URI=redis://[HOST]:[PORT] \
#     -e ADDRESS=0.0.0.0:8080 (Port same as in -p variable)
#     nnn-rust-image

FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --locked --bin nnn-rust-back

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/nnn-rust-back /usr/local/bin

EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/nnn-rust-back"]
