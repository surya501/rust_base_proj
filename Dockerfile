FROM lukemathwalker/cargo-chef:latest-rust-1.63.0 as chef
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef as planner
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json
# Up to this point, if our dependency tree stays the same,
# all layers should be cached.
# copy the source code into the container
COPY . .

# We dont want to contact the postgres server to validate
# the schema during the build
ENV SQLX_OFFLINE true

# build the project
RUN cargo build --release --bin base_proj

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