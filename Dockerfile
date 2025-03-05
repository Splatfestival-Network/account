FROM rust:1.85 as builder

WORKDIR /app

# this looks like being stupid, but docker cache is FIRE once you do this
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo fetch

# there's the beauty
COPY . .

RUN cargo build --release

FROM rust:1.85 AS final

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/account /app/account

# Set executable permissions
RUN chmod +x /app/eshop-rs

# Command to run the application
CMD ["ls /app"]
CMD ["/app/account"]
