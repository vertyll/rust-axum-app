# Rust Axum App

A web server application built with Rust and Axum framework.

## Description

This project implements a simple HTTP server using the Axum web framework for Rust.

## Features

- REST API with Axum
- Asynchronous runtime with Tokio
- JSON serialization with Serde

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

First, install cargo-watch to REPL.

```bash
# Install cargo-watch
cargo install cargo-watch
```

Second, run the server in REPL mode.

````bash
# To run the server.
cargo watch -q -c -w src/ -x "run"
````

### Dev

Run the server.

```bash
# To run the server.
cargo run
```