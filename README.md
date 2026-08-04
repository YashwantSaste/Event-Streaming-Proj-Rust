# Event Streaming Project in Rust

A Rust workspace that implements a lightweight event streaming system with a **broker**, **producer**, **consumer**, shared protocol/types, and a small **CLI** for local workflows.

## Overview

This repository is organized as a Cargo workspace with the following crates:

- **broker** – accepts requests, manages topics/partitions, and stores records
- **producer** – creates topics, lists topics, and publishes records to the broker
- **consumer** – subscribes to topics, polls records, and commits offsets
- **common** – shared configuration, models, protocol, logging, filesystem, and error types
- **cli** – local command runner that wraps common operations such as workspace init, broker start, topic creation, producing, and consuming

The project appears to be designed as a simplified Kafka-like event streaming system for learning and experimentation in Rust.

## Workspace Structure

```text
.
├── assets/
│   └── broker.toml
├── broker/
├── cli/
├── common/
├── consumer/
├── producer/
├── Cargo.toml
└── Cargo.lock
```

Root `Cargo.toml` defines a workspace with these members:

- `broker`
- `consumer`
- `producer`
- `common`
- `cli`

## Components

### Broker

The broker starts an async Tokio application and loads configuration from a TOML file.

Default config path:

- `assets/broker.toml`

Configurable with:

- `--config <path>`

The broker is responsible for:

- network request handling
- topic and partition management
- storage management
- consumer group coordination
- reading broker configuration from TOML

### Producer

The producer connects to the broker and supports:

- **create-topic**
- **list-topics**
- **produce**

It includes retry behavior for broker communication.

### Consumer

The consumer connects to the broker and supports:

- subscribing to a topic
- polling records from a partition and offset
- optionally committing offsets for a consumer group

### Common

The shared crate contains common building blocks used across the workspace:

- configuration
- constants
- error types
- filesystem abstraction
- logging
- models
- wire protocol

### CLI

The CLI crate provides higher-level commands for local development workflows. Based on the source structure, it includes commands for:

- initializing a workspace
- starting the broker
- creating a topic
- listing topics
- producing messages
- consuming messages

## Prerequisites

- **Rust** toolchain installed
- **Cargo** available in your shell

Recommended:

- latest stable Rust via [rustup](https://rustup.rs/)

## Build

Build the entire workspace:

```bash
cargo build
```

Build in release mode:

```bash
cargo build --release
```

## Run

### Start the broker

Run the broker with the default configuration:

```bash
cargo run -p broker
```

Run the broker with a custom config file:

```bash
cargo run -p broker -- --config path/to/broker.toml
```

### Producer commands

Create a topic:

```bash
cargo run -p producer -- \
  --command create-topic \
  --topic demo \
  --partitions 1 \
  --segment-max-bytes 1048576
```

List topics:

```bash
cargo run -p producer -- \
  --command list-topics
```

Produce a message:

```bash
cargo run -p producer -- \
  --command produce \
  --topic demo \
  --partition 0 \
  --message "hello world"
```

Produce a message with a key:

```bash
cargo run -p producer -- \
  --command produce \
  --topic demo \
  --partition 0 \
  --key user-1 \
  --message "event payload"
```

Optional producer flags:

- `--host` (default: `127.0.0.1`)
- `--port` (default: `9092`)
- `--partition` (default: `0`)

### Consumer commands

Consume records:

```bash
cargo run -p consumer -- \
  --topic demo \
  --partition 0 \
  --offset 0 \
  --max-records 10
```

Consume and commit the next offset:

```bash
cargo run -p consumer -- \
  --topic demo \
  --group demo-group \
  --partition 0 \
  --offset 0 \
  --max-records 10 \
  --commit true
```

Optional consumer flags:

- `--host` (default: `127.0.0.1`)
- `--port` (default: `9092`)
- `--group` (default: `default`)
- `--partition` (default: `0`)
- `--offset` (default: `0`)
- `--max-records` (default: `10`)

## Broker Configuration

Default broker configuration file:

```toml
[broker]
host = "127.0.0.1"
port = 9092

[storage]
data_directory = "data/broker"
topics_directory = "data/topics"
consumer_group_directory = "data/consumer-groups"
segment_max_bytes = 1048576

[network]
max_frame_bytes = 1048576

[logging]
level = "info"
file = "logs/broker.log"
```

### Configuration fields

#### `[broker]`

- `host` – interface/address the broker binds to
- `port` – broker port

#### `[storage]`

- `data_directory` – base broker data directory
- `topics_directory` – topic storage location
- `consumer_group_directory` – consumer group metadata location
- `segment_max_bytes` – maximum segment size

#### `[network]`

- `max_frame_bytes` – maximum allowed request/response frame size

#### `[logging]`

- `level` – log level
- `file` – log output file path

## Example Local Flow

1. Start the broker:

```bash
cargo run -p broker
```

2. Create a topic:

```bash
cargo run -p producer -- \
  --command create-topic \
  --topic demo \
  --partitions 1
```

3. Produce a few messages:

```bash
cargo run -p producer -- \
  --command produce \
  --topic demo \
  --partition 0 \
  --message "first event"
```

```bash
cargo run -p producer -- \
  --command produce \
  --topic demo \
  --partition 0 \
  --message "second event"
```

4. Consume messages:

```bash
cargo run -p consumer -- \
  --topic demo \
  --partition 0 \
  --offset 0 \
  --max-records 10
```

## Development Notes

- The broker uses **Tokio** for async runtime support.
- The producer includes a retry policy for broker communication.
- The consumer supports explicit offset commit behavior by consumer group.
- Shared protocol and models are centralized in the `common` crate.

## Repository Branch

The linked branch for this work is:

- `broker-impl`

## Future Improvements

Some useful additions for future iterations:

- tests and integration test examples
- protocol documentation
- message schema/versioning docs
- consumer group rebalance documentation
- persistence/storage format documentation
- Docker setup for local runs
- benchmark results and performance notes

## License

No license file is currently present in the repository. Add a `LICENSE` file if you want to define usage terms explicitly.