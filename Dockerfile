FROM rust:latest as builder
RUN apt-get update
RUN cd /tmp && USER=root cargo new --bin dt-instance
WORKDIR /tmp/dt-instance

# Build Rust skeleton project, caching dependencies, before building.
COPY Cargo.toml Cargo.lock ./
RUN touch build.rs && echo "fn main() {println!(\"cargo:rerun-if-changed=\\\"/tmp/dt-instance/build.rs\\\"\");}" >> build.rs
RUN cargo build --release

# Force the build.rs script to run by modifying it
RUN echo " " >> build.rs
COPY ./src ./src
RUN cargo build --release

# Push built release to slim container
FROM debian:buster-slim
COPY --from=builder /tmp/dt-instance/target/release/dt-instance /usr/local/bin/dt-instance
COPY wait-for wait-for
