FROM rustembedded/cross:x86_64-unknown-linux-gnu

RUN \
    git clone https://github.com/pothosware/SoapySDR.git &&\
    cd SoapySDR &&\
    git checkout soapy-sdr-0.8.1 &&\
    mkdir build &&\
    cd build &&\
    cmake -D CMAKE_INSTALL_PREFIX=/ .. &&\
    make -j4 &&\
    make install

RUN yum update -y && \
    yum install centos-release-scl -y && \
    yum install llvm-toolset-7 -y

ENV LIBCLANG_PATH=/opt/rh/llvm-toolset-7/root/usr/lib64/ \
    LIBCLANG_STATIC_PATH=/opt/rh/llvm-toolset-7/root/usr/lib64/ \
    CLANG_PATH=/opt/rh/llvm-toolset-7/root/usr/bin/clang

