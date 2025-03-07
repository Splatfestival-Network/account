FROM rust:alpine as builder

RUN apk add --no-cache musl-dev openssl-dev musl openssl libcrypto3

WORKDIR /app

COPY . .

RUN cargo build --profile prod

RUN rm .env

FROM rust:alpine AS final

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/prod/account /app/account

# Set executable permissions
RUN chmod +x /app/account

# Command to run the application
CMD ["ls /app"]
CMD ["/app/account"]
