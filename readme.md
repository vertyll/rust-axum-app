# Rust Axum App

A web server application built with Rust and Axum framework.

## Description

This project implements a simple HTTP server using the Axum web framework for Rust.

## Features

- REST API with Axum
- Asynchronous runtime with Tokio
- JSON serialization with Serde
- SeaORM - Async ORM for Rust

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

## Development

### Dev (REPL)

First, install the sea-orm-cli with cargo.

```bash
cargo install sea-orm-cli@1.1.0
```

Second, install cargo-watch to REPL.

```bash
cargo install cargo-watch
```

Third, run the server in REPL mode.

````bash
cargo watch -q -c -w src/ -x "run"
````

### Dev

Run the server.

First, install the sea-orm-cli with cargo.

```bash
cargo install sea-orm-cli@1.1.0
```

Second, run the server.

```bash
cargo run
```