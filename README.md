# account.rs

Reimplementation of the Nintendo Network Account System (NNAS) in Rust.

# Requirements

- Rust (rustup recommended)
- A PostgreSQL database
- An S3 bucket
- An SMTP server

For more info on configuring the server, check the .env.example file, it has comments that should help.

# Building

This step shouldn't be required once we start making official releases, but for now you need to build it yourself.
You will need to set your database's connection URL in the environment variable DATABASE_URL with `export`. Without it, the server will not build.

```bash
export DATABASE_URL=postgresql://your-user:your-password@your-postgresql-server:5432/your-db
```

Once you have that set up, you should just be able to run `cargo build` and it will build to `target/account-server-rust`.

