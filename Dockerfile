FROM rust:1.77-alpine AS rust

RUN apk add musl-dev openssl-dev

WORKDIR /app

# Cache dependencies: https://benjamincongdon.me/blog/2019/12/04/Fast-Rust-Docker-Builds-with-cargo-vendor/
COPY Cargo.toml Cargo.lock ./

RUN mkdir .cargo
RUN cargo vendor --locked > .cargo/config

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
      tini openssl libgcc \
    && rm -rf /var/cache/* \
    && mkdir /var/cache/apk
WORKDIR /app
COPY --from=rust /app/main ./

ENTRYPOINT ["/sbin/tini", "--"]
CMD ["/app/main"]

EXPOSE 80
