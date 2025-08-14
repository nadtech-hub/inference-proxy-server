FROM rust:latest

WORKDIR /usr/src/server

COPY . .

RUN cargo install --path . 

RUN cargo build --release

CMD ["inference-proxy-server"]
