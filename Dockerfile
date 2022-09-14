FROM rust:latest AS build
WORKDIR /app
RUN cargo init .
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release
COPY . .
RUN cargo test
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM debian:bullseye
COPY --from=build /app/target/x86_64-unknown-linux-musl/release/fluent_data /usr/local/bin
CMD [ "fluent_data", "--service" ]