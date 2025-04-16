# storage-app

## Overview

This project takes heavily inspiration from [Seafile](https://www.seafile.com/) in its functionality and UI design. File hosting service with performance, ease of use, and being lightweight.

Key features include:

- Web UI for managing files, with minimal/no client-side javascript
- Multiple storage backends supported (local filesystem, S3, etc)
- Multiple libraries per user, each with configurable storage backends
- WebDAV (soon)


## Getting Started

### Prerequisites

- Rust 1.86+ (stable) and Cargo
- PostgreSQL 13+ database
- 512MB RAM minimum

### Installing

```bash
# Clone the repository
git clone https://github.com/jackzmc/storage.git
cd storage

# Configure your database (create .env file with your PostgreSQL connection)
echo "DATABASE_URL=postgres://username:password@localhost" > .env

# Build the project
cargo build --release

# Run database migrations
#cargo run --bin migrate --features migrations

# Run the server
cargo run --release
```

## Roadmap

* [ ] WebDAV Support
* [ ] S3 backend support
* [ ] Administration panel
  * [ ] Add storage backends
  * [ ] Manage users
  * [ ] Change app settings
* [ ] Email support (for password resets, user invites)

## Documentation

Todo, will be available at https://git.jackz.me/jackz/storage/wiki

# License

Available under the AGPL-3.0 license. [LICENSE](LICENSE)