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
FROM debian:bullseye-slim AS runtime

WORKDIR /app

# Install OpenSSL - it is dynamically linked by some of our dependencies
# Install ca-certificates - it is needed to verify TLS certificates
# when establishing HTTPS connections
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*


# copy the binary from the builder stage
COPY --from=builder /app/target/release/base_proj base_proj
COPY configuration configuration
ENV APP_ENVIRONMENT production

# run the project
ENTRYPOINT [ "./base_proj" ]