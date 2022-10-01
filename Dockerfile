FROM rust:latest as builder

WORKDIR /usr/src/botofgreed-rs
# copy over project
COPY . .
# install cmake (required for building snmalloc)
RUN apt update
RUN apt install cmake -y
# install executable
RUN cargo install --path .

# and run it (environment vars must be set in docker-compose.yml)
CMD ["botofgreed-rs"]
