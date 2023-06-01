FROM rust:1.69 as build

RUN apt-get update \
    && apt-get -y install build-essential \
    && apt-get -y install libclang-dev \
    && apt-get install -y cmake 

RUN git clone https://github.com/rconan/dos-actors.git 
WORKDIR /dos-actors
RUN git submodule update --init

RUN cargo build --release --package asms --bin trfun-an --features trfun-an

FROM rust:1.69-slim

COPY --from=build /dos-actors/target/release/trfun-an trfun-an