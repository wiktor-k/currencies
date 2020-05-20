FROM rust as cargo-build

WORKDIR /usr/src/app
COPY Cargo.lock Cargo.toml ./
COPY ./src src
RUN cargo build --release

EXPOSE 3000

CMD ["/usr/src/app/target/release/currencies"]
