FROM rust:1.63.0 as builder

# Docker will create the app folder if it doesn't exist
WORKDIR /app

# install required dependencies
RUN apt update && apt install -y lld clang

# copy the source code into the container
COPY . .

# We dont want to contact the postgres server to validate
# the schema during the build
ENV SQLX_OFFLINE true

# build the project
RUN cargo build --release

# Let's create a new stage for runtime;
#  this will be a smaller image (ideally!!)
FROM rust:1.63.0 as runtime

WORKDIR /app

# copy the binary from the builder stage
COPY --from=builder /app/target/release/base_proj base_proj
COPY configuration configuration
ENV APP_ENVIRONMENT production

# run the project
ENTRYPOINT [ "./base_proj" ]