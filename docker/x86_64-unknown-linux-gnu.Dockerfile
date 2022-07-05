FROM rust:1.59-slim-buster

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
