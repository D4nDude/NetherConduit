FROM rust:1.98.0 AS chef

RUN cargo install --locked cargo-chef
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml .
COPY Cargo.lock .
COPY ./crates ./crates
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY Cargo.toml .
COPY Cargo.lock .
COPY ./crates ./crates
RUN cargo build --release --bin netherconduit

FROM debian:trixie-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/netherconduit /usr/local/bin

EXPOSE 25565

ENTRYPOINT [ "/usr/local/bin/netherconduit"]