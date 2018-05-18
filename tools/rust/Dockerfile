FROM ubuntu:16.04
ARG RUST_VERSION=1.26.0

MAINTAINER Sehyo Chang "sehyo@nginx.com"

# install nginx required libraries

RUN apt-get update

# install dev en
RUN apt-get install vim autoconf automake libtool curl make wget  unzip gnupg binutils pkg-config -y

# essential c compiler toolchain includes automake
#n RUN apt-get install build-essential -y
# install clang
RUN wget -O - http://apt.llvm.org/llvm-snapshot.gpg.key | apt-key add -
RUN echo 'deb http://apt.llvm.org/xenial/ llvm-toolchain-xenial main' >> /etc/apt/sources.list
RUN wget -O - http://apt.llvm.org/llvm-snapshot.gpg.key | apt-key add -
RUN apt-get update
RUN apt-get install llvm-3.9-dev libclang-3.9-dev clang-3.9 -y

# This is special hack to comment out IPPORT_RESERVED because it conflicts with in.h
# which prevents from compilation of rust binding.rs
RUN sed -i 's:# define IPPORT_RESERVED:// #define IPPORT_RESERVED:' /usr/include/netdb.h

# install PCRE library
RUN apt-get install libpcre3 libpcre3-dev zlib1g-dev libssl-dev -y


# install rust
RUN curl -sO https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init && \
  chmod +x rustup-init && \
  ./rustup-init -y --default-toolchain $RUST_VERSION --no-modify-path && \
  rm -rf \
    rustup-init \
    /var/lib/apt/lists/* \
    /tmp/* \
    /var/tmp/* 


ENV PATH $PATH:/root/.cargo/bin



# install protobuf
RUN curl -OL https://github.com/google/protobuf/releases/download/v3.3.0/protoc-3.3.0-linux-x86_64.zip
RUN unzip protoc-3.3.0-linux-x86_64.zip -d protoc3
RUN mv protoc3/bin/* /usr/local/bin/
RUN rm protoc-3.3.0-linux-x86_64.zip

