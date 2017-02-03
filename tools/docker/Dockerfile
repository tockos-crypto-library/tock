FROM ubuntu:16.04

# Install some needed packages
RUN apt -y update
RUN apt -y install software-properties-common curl wget build-essential git

# Need arm-none-eabi-gcc
RUN add-apt-repository ppa:team-gcc-arm-embedded/ppa
RUN apt-key adv --keyserver keyserver.ubuntu.com --recv-keys D1FAA6ECF64D33B0
RUN apt -y update
RUN apt -y install gcc-arm-embedded

# Setup Rust
#RUN curl -sO https://static.rust-lang.org/dist/rust-$RUST_VERSION-x86_64-unknown-linux-gnu.tar.gz
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
#RUN wget https://sh.rustup.rs -O rustup.sh
#RUN chmod 755 rustup.sh
#RUN ./rustup.sh -y

#RUN export PATH=$HOME/.cargo/bin:$PATH

ENV PATH="/root/.cargo/bin:${PATH}"

RUN cargo install rustfmt

RUN git clone https://github.com/helena-project/tock





