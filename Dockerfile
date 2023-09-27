# This Dockerfile is used to build a socks proxy server.
FROM rust:1.72 as build

RUN rustup component add rustfmt

RUN apt-get update && apt-get install -y \
    cmake \
 && rm -rf /var/lib/apt/lists/*

# Copy over relevant crates
COPY ./socksx /socksx

# Build an optimized binary
WORKDIR /socksx
RUN cargo build --release

# Define final image
FROM ubuntu:23.10

RUN apt-get update && apt-get install -y \
    libssl3 \
    libuv1 \
 && rm -rf /var/lib/apt/lists/*

# Copy `socksx` from the build stage
COPY --from=build /socksx/target/release/socksx .

EXPOSE 1080
ENTRYPOINT [ "./socksx" ]
