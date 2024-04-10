FROM --platform=$BUILDPLATFORM rust:1.77 AS rust

# cross-compile using clang/llvm: https://github.com/briansmith/ring/issues/1414#issuecomment-1055177218
RUN apt-get update && apt-get -y install libssl-dev musl-tools clang llvm gcc-multilib

ENV CC_aarch64_unknown_linux_musl=clang
ENV AR_aarch64_unknown_linux_musl=llvm-ar
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-Clink-self-contained=yes -Clinker=rust-lld"

ARG TARGETPLATFORM
RUN case "$TARGETPLATFORM" in \
      "linux/arm64") echo aarch64-unknown-linux-musl > /target ;; \
      "linux/amd64") echo x86_64-unknown-linux-musl > /target ;; \
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
      tini \
    && rm -rf /var/cache/* \
    && mkdir /var/cache/apk
WORKDIR /app
COPY --from=rust /app/main ./

ENTRYPOINT ["/sbin/tini", "--"]
CMD ["/app/main"]

EXPOSE 80
