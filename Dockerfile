FROM rust:1.63.0

# Docker will create the app folder if it doesn't exist
WORKDIR /app

# install required dependencies
RUN apt update && apt install -y lld clang

# copy the source code into the container
COPY . .

# We dont want to contact the postgres server to validate
# the schema during the build
ENV SQLX_OFFLINE true
ENV APP_ENVIRONMENT production

# build the project
RUN cargo build --release

# run the project
ENTRYPOINT [ "./target/release/base_proj" ]