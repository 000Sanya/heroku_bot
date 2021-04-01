cp ./target/release/heroku_bot ./docker/
cd docker
sudo docker build -t heroku_bot_t .