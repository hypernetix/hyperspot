# ModKit DB

Database abstractions for CyberFabric / ModKit with optional SeaORM integration.

## Overview

The `cf-modkit-db` crate provides:

- Typed database configuration / connection options
- SQLx backend support (SQLite / Postgres / MySQL via features)
- SeaORM integration
- Secure-by-default ORM wrapper (see `secure` module)

## Features

- `pg`, `mysql`, `sqlite`: enable SQLx backends
- `insecure-escape`: escape hatch for administrative operations (off by default)

## License

Licensed under Apache-2.0.
