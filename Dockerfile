FROM --platform=$BUILDPLATFORM rust:1.77 AS rust

RUN apt-get update && apt-get -y install libssl-dev clang llvm

ARG TARGETPLATFORM
RUN case "$TARGETPLATFORM" in \
      "linux/arm64") echo aarch64-unknown-linux-gnu > /target ;; \
      "linux/amd64") echo x86_64-unknown-linux-gnu > /target ;; \
      *) echo Unsupported architecture && exit 1 ;; \
    esac

RUN rustup target add $(cat /target)
RUN rustup component add rustfmt clippy

WORKDIR /app

# Cache dependencies: https://benjamincongdon.me/blog/2019/12/04/Fast-Rust-Docker-Builds-with-cargo-vendor/
COPY Cargo.toml Cargo.lock ./

RUN mkdir .cargo
RUN cargo vendor --locked > .cargo/config

COPY src ./src

# Check code quality in one step
RUN cargo fmt --all -- --check && \
  cargo clippy --locked -- -D warnings

ENV SOURCE_DATE_EPOCH=1
RUN cargo build --locked --release --target $(cat /target)

RUN cp target/$(cat /target)/release/main .

RUN sha256sum main

FROM alpine:3.19
ENV \
    # Show full backtraces for crashes.
    RUST_BACKTRACE=full
RUN apk add --no-cache \
      tini openssl \
    && rm -rf /var/cache/* \
    && mkdir /var/cache/apk
WORKDIR /app
COPY --from=rust /app/main ./

ENTRYPOINT ["/sbin/tini", "--"]
CMD ["/app/main"]

EXPOSE 80
