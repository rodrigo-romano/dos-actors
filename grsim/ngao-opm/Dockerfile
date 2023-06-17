FROM rust:1.70 as build

RUN apt-get update \
    && apt-get -y install build-essential \
    && apt-get -y install libclang-dev \
    && apt-get install -y cmake gcc noweb

RUN wget -q https://developer.download.nvidia.com/compute/cuda/12.1.1/local_installers/cuda_12.1.1_530.30.02_linux.run
RUN sh cuda_12.1.1_530.30.02_linux.run --silent --toolkit --no-drm

RUN git clone https://github.com/rconan/dos-actors.git 
COPY modal_state_space_model_2ndOrder.zip /dos-actors
WORKDIR /dos-actors
RUN git submodule update --init

COPY . /dos-actors/grsim/ngao-opm
RUN CUDACXX=/usr/local/cuda-12.1/bin/nvcc FEM_REPO=/dos-actors \
    cargo build  --release --package ngao-opm --bin ngao-opm
RUN CUDACXX=/usr/local/cuda-12.1/bin/nvcc FEM_REPO=/dos-actors \
    cargo build  --release --package ngao-opm --bin ngao-opm-domeseeing --features domeseeing
RUN CUDACXX=/usr/local/cuda-12.1/bin/nvcc FEM_REPO=/dos-actors \
    cargo build  --release --package ngao-opm --bin ngao-opm-windloading --features windloading

FROM nvidia/cuda:12.1.1-runtime-ubuntu20.04

COPY --from=build /dos-actors/target/release/ngao-opm ngao-opm
COPY --from=build /dos-actors/target/release/ngao-opm-domeseeing ngao-opm-domeseeing
COPY --from=build /dos-actors/target/release/ngao-opm-windloading ngao-opm-windloading