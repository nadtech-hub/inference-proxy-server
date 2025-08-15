FROM rust:latest

WORKDIR /usr/src/server

COPY . .

RUN cargo install --path . 

CMD ["inference-proxy-server"]
