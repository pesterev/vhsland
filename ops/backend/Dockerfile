FROM rust

WORKDIR /usr/src/vhsland

COPY . .

RUN cargo build --release

RUN cargo install --path .

CMD ["/usr/local/cargo/bin/vhsland"]

EXPOSE 3000