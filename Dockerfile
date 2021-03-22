FROM rust:1.46 as builder
COPY . .
RUN cargo build --release

FROM debian:buster-slim
RUN apt-get update && apt-get upgrade -y && apt-get autoremove --yes && rm -rf /var/lib/apt/lists/*
COPY --from=builder target/release/list /usr/local/bin/list
CMD ["list"]
