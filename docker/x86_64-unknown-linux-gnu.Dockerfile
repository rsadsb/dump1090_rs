FROM ghcr.io/cross-rs/x86_64-unknown-linux-gnu:main

ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get update -y && apt-get install -y cmake git llvm-dev libclang-dev clang pkg-config

RUN \
    git clone https://github.com/pothosware/SoapySDR.git &&\
    cd SoapySDR &&\
    git checkout soapy-sdr-0.8.1 &&\
    mkdir build &&\
    cd build &&\
    cmake -D CMAKE_INSTALL_PREFIX=/ .. &&\
    make -j4 &&\
    make install

ENV LD_LIBRARY_PATH=/usr/local/lib
