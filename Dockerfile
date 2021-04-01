FROM debian:buster

RUN apt-get update && \
    apt-get install -y libssl1.1 ca-certificates
RUN update-ca-certificates
COPY target/release/heroku_bot /bot
RUN chown -R 1000:1000 /bot
USER 1000
ENTRYPOINT ["/bot/heroku_bot"]