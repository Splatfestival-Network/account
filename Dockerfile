FROM rust:1.85 as builder

WORKDIR /app

COPY . .

RUN cargo build --release

RUN rm .env

FROM rust:1.85 AS final

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/account /app/account

# Set executable permissions
RUN chmod +x /app/account

# Command to run the application
CMD ["ls /app"]
CMD ["/app/account"]
