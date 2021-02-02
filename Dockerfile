FROM rust:1.49 AS build

WORKDIR /usr/src/heroku_app
COPY Cargo.toml Cargo.lock ./
COPY src/  ./src
COPY pixiv_api/  ./pixiv_api
RUN cargo build --release
RUN ls -la target/release

FROM debian:buster

RUN apt-get update; \
    apt-get install -y libssl1.1 ca-certificates
RUN update-ca-certificates
COPY --from=build /usr/src/heroku_app/target/release/heroku_bot .
USER 1000
CMD ["./heroku_bot"]