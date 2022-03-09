FROM --platform=$BUILDPLATFORM rust:1.59.0 AS rust

# cross-compile using clang/llvm: https://github.com/briansmith/ring/issues/1414#issuecomment-1055177218
RUN apt-get update && apt-get -y install musl-tools clang llvm

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

WORKDIR /app

# Cache dependencies: https://benjamincongdon.me/blog/2019/12/04/Fast-Rust-Docker-Builds-with-cargo-vendor/
COPY Cargo.toml Cargo.lock ./

RUN mkdir .cargo
RUN cargo vendor > .cargo/config

COPY src ./src
RUN cargo build --release --target $(cat /target)

RUN cp target/$(cat /target)/release/main .

FROM alpine:3.15
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
