FROM rust:1.77-alpine AS rust

# cross-compile using clang/llvm: https://github.com/briansmith/ring/issues/1414#issuecomment-1055177218
#RUN dpkg --add-architecture arm64
#RUN apt-get update && apt-get -y install libssl-dev:arm64 openssl:arm64 musl-tools clang llvm pkg-config

RUN apk add --no-cache musl-dev clang llvm openssl-dev openssl pkgconfig

#RUN rustup component add rustfmt clippy

WORKDIR /app

# Cache dependencies: https://benjamincongdon.me/blog/2019/12/04/Fast-Rust-Docker-Builds-with-cargo-vendor/
COPY Cargo.toml Cargo.lock ./

#RUN mkdir .cargo
#RUN cargo vendor --locked > .cargo/config

COPY src ./src

# Check code quality in one step
#RUN cargo fmt --all -- --check && \
#  cargo clippy --locked -- -D warnings

ENV SOURCE_DATE_EPOCH=1
RUN RUSTFLAGS="-Ctarget-feature=-crt-static" cargo build --release

RUN cp target/release/main .

RUN sha256sum main

FROM alpine:3.19
ENV \
    # Show full backtraces for crashes.
    RUST_BACKTRACE=full
RUN apk add --no-cache \
      tini \
    && rm -rf /var/cache/* \
    && mkdir /var/cache/apk
WORKDIR /app
COPY --from=rust /app/main ./

ENTRYPOINT ["/sbin/tini", "--"]
CMD ["/app/main"]

EXPOSE 80
