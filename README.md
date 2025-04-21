# storage-app

(needs a better name)

## Overview

This project takes heavily inspiration from [Seafile](https://www.seafile.com/) in its functionality and UI design. File hosting service with performance, ease of use, and being lightweight.

**Notice: Project is under heavy active development**

Key features include:

- Web UI for managing files, with minimal/no client-side javascript
- Multiple storage backends supported (local filesystem, S3, etc)
- Multiple libraries per user, each with configurable storage backends
- WebDAV (soon)

![screenshot of files screen](docs/images/screenshot-files.png)
_The current files list when logged in_

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

# Copy the sample config
cp config.sample.toml config.toml
# Edit the config.toml or provide the equivalant settings with env
# ex: [auth.oidc]  ---> STORAGE_auth.oidc.issuer__url
#     issuer-url
# Configure your database (requires to be set by env for now)
echo "DATABASE_URL=postgres://username:password@localhost" > .env

# Build the project
cargo build --release

# Run database migrations
#cargo run --bin migrate --features migrations

# Run the server
cargo run --release
```

## Roadmap

Rough roadmap in a rough order of priority

* [ ] WebDAV Support
* [ ] Email support (for password resets, user invites)
  * [ ] Email sender utility
  * [ ] Individual email actions
* [ ] SSO Support (openid)
  * [x] Basic implementation
  * [ ] User mapping
  * [ ] User creation
  * [ ] User logout
* [ ] S3 backend support
* [ ] Administration panel
  * [ ] Add storage backends
  * [ ] Manage users
  * [ ] Change app settings
  
## Documentation

Todo, will be available at https://git.jackz.me/jackz/storage/wiki

## Development

The server is written in Rust, using the [Rocket](https://rocket.rs/) framework and [handlebars](https://github.com/sunng87/handlebars-rust) for HTML templating.
The frontend is just HTML and CSS, using the [Bulma CSS framework](https://bulma.io). As little javascript as possible should be done for frontend work.

Contributions are welcome

# License

Available under the AGPL-3.0 license. [LICENSE](LICENSE)