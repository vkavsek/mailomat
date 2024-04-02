FROM lukemathwalker/cargo-chef:latest-rust-1.77 AS chef
WORKDIR /app

RUN apt update && apt install mold clang -y 

########################################

FROM chef AS planner

# Copy contents of current DIR to the image
COPY . .
# Compute a lock-like file for our project 
RUN cargo chef prepare --recipe-path recipe.json

########################################

FROM chef AS builder 

COPY --from=planner /app/recipe.json recipe.json
# Build our project dependecies, not our app!
RUN cargo chef cook --release --recipe-path recipe.json
# If our dependency tree hasn't changed, everything should be cached up to now!
COPY . .

RUN cargo build --release --bin mailer


########################################

FROM debian:bookworm-slim AS runtime 
WORKDIR /app

# Install OpenSSL - it's dynamically linked by some of our dependencies.
# Install ca-certificates - it's needed to verify TLS certificates when establishing HTTPS con.
RUN apt-get update -y \
	&& apt-get install -y --no-install-recommends openssl ca-certificates \
	&& apt-get autoremove -y \
	&& apt-get clean -y \
	&& rm -rf /var/lib/apt/lists/*


# Copy the compiled binary from builder to runtime.
COPY --from=builder /app/target/release/mailer mailer
# config/ is needed at runtime!
COPY config config
ENV APP_ENVIRONMENT production
ENV RUST_LOG info

ENTRYPOINT [ "./mailer" ]
