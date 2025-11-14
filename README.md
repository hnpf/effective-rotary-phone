# Async Net Program

[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

A little Rust program with async networking! This project is a simple async TCP server and client with echo and ping functionality, using a custom-built async runtime.

## Description

The server listens on `127.0.0.1:8080` and echoes back any data it receives from connected clients. It's a basic demonstration of async I/O in Rust.

## Features

- Async TCP server and client
- A custom async runtime (built from scratch, no Tokio!!!)
- Concurrent client handling with threads
- Echo and ping funcs

## How to Run

1. Make sure you have Rust installed (version 1.70 or later).
2. Clone or download this project.
3. Navigate to the project directory.
4. Run `cargo run --bin server` to start the server.
5. In another terminal, run `cargo run --bin client` to start the client.
6. Type messages in the client terminal and see them echoed back, with periodic pings.

## License


This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
