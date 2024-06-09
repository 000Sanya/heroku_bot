FROM rust:1.75.0-alpine3.19 as builder
RUN apk add musl-dev openssl-dev
WORKDIR /app/bot
COPY Cargo.toml /app/bot/
COPY Cargo.lock /app/bot/
COPY src /app/bot/src

RUN cargo build --release

FROM alpine:3.19
WORKDIR /app

EXPOSE 8080

COPY --from=builder /app/bot/target/release/heroku_bot ./

CMD ["/app/heroku_bot"]