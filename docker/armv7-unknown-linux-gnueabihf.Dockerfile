FROM rustembedded/cross:armv7-unknown-linux-gnueabihf-0.2.1

RUN \
    git clone https://github.com/pothosware/SoapySDR.git &&\
    cd SoapySDR &&\
    git checkout soapy-sdr-0.8.1 &&\
    mkdir build &&\
    cd build &&\
    cmake \
        -D CMAKE_C_COMPILER=/usr/bin/arm-linux-gnueabihf-gcc \
        -D CMAKE_CXX_COMPILER=/usr/bin/arm-linux-gnueabihf-g++ \
        -D CMAKE_AR=/usr/bin/arm-linux-gnueabihf-ar \
        -D CMAKE_C_COMPILER_AR=/usr/bin/arm-linux-gnueabihf-gcc-ar \
        -D CMAKE_C_COMPILER_RANLIB=/usr/bin/arm-linux-gnueabihf-gcc-ranlib \
        -D CMAKE_LINKER=/usr/bin/arm-linux-gnueabihf-ld \
        -D CMAKE_NM=/usr/bin/arm-linux-gnueabihf-nm \
        -D CMAKE_OBJCOPY=/usr/bin/arm-linux-gnueabihf-objcopy \
        -D CMAKE_OBJDUMP=/usr/bin/arm-linux-gnueabihf-objdump \
        -D CMAKE_RANLIB=/usr/bin/arm-linux-gnueabihf-ranlib \
        -D CMAKE_STRIP=/usr/bin/arm-linux-gnueabihf-strip .. &&\
    make -j4 &&\
    make install &&\
    ldconfig

RUN apt-get install -y libclang-dev

ENV LD_LIBRARY_PATH=/usr/local/lib
