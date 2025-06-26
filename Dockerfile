FROM rust:1.87-slim AS builder

WORKDIR /usr/src/goeie-server

COPY src src
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

RUN cargo build --release


FROM rust:1.86.0-slim AS runner

WORKDIR /usr/src/goeie
COPY --from=builder /usr/src/goeie-server/target/release/goeie-server goeie-server

LABEL org.opencontainers.image.source=https://github.com/borisnliscool/goeie

CMD ["/bin/sh", "-c", "./goeie-server"]