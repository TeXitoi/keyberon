FROM rust:slim
RUN apt-get update
RUN rustup target add thumbv7m-none-eabi
RUN apt-get install -y git gdb-arm-none-eabi openocd vim make openscad
COPY . .
RUN cargo build --release --example keyberon60

entrypoint /bin/bash
