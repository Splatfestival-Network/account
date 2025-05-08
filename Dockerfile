# syntax=docker/dockerfile:1

FROM rust:alpine as builder

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static protobuf-dev lld

WORKDIR /app

# this optimizes build time by putting the dependencies in a seperate docker layer, speeding up future builds
COPY mii ./mii
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo fetch

COPY . .

RUN OPENSSL_LIB_DIR=/usr/lib OPENSSL_INCLUDE_DIR=/usr/include/openssl OPENSSL_STATIC=1 RUSTFLAGS="-C target-feature=+aes,+sse -C relocation-model=static -C linker=ld.lld" cargo build --profile prod --target x86_64-unknown-linux-musl

RUN rm .env

FROM scratch AS final

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-musl/prod/account /account

# Set executable permissions
# RUN chmod +x /account

# Command to run the application
ENTRYPOINT ["/account"]
