# Rust Axum App

A web server application built with Rust and Axum framework.

## Description

This project implements a simple HTTP server using the Axum web framework for Rust.

## Features

- REST API with Axum
- Asynchronous runtime with Tokio
- JSON serialization with Serde
- SeaORM - Async ORM for Rust
- cargo-watch for REPL
- rust-i18n for internationalization
- PostgreSQL for database

## Prerequisites

- Rust 2024 edition
- Cargo package manager

## Installation

Clone the repository and build the project:

```bash
git clone <repository-url>
cd rust-axum-app
cargo build
```

## Development (without Docker/Podman)

### Configuration environment:

1. Install PostgreSQL and create a database: `rust_axum_app`
2. Copy the `.env.example` file to `.env` and set the database connection string.

```bash
cp .env.example .env
```

### Dev (REPL)

Install the sea-orm-cli with cargo.

```bash
cargo install sea-orm-cli@1.1.0
```

Install cargo-watch to REPL.

```bash
cargo install cargo-watch
```

Run the server in REPL mode.

````bash
cargo watch -q -c -w src/ -x "run"
````

### Dev

Install the sea-orm-cli with cargo.

```bash
cargo install sea-orm-cli@1.1.0
```

Run the server.

```bash
cargo run
```