FROM rust:1.77

RUN apt update && apt install mold clang -y 

WORKDIR /app
# Copy contents of current DIR to the image
COPY . .
RUN cargo build --release

ENTRYPOINT [ "./target/release/mailer" ]
