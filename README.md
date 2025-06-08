# grapple_db

`grapple_db` is a library designed to simplify interactions with various databases. It offers a flexible architecture that allows you to select the database client you need and utilize it according to your requirements. The library can be easily integrated into your project using feature flags.

## Table of Contents

- [grapple_db](#grapple_db)
- [Description](#description)
- [Features](#features)
- [Installation](#installation)
- [Redis Examples](#redis-examples)
  - [Create Client](#create-client)
  - [Define Model](#define-model)
  - [Get](#get)
  - [Set](#set)
  - [Del](#del)
  - [Other](#other)
  - [Not Covered Methods](#not-covered-methods)
- [Scylla Examples](#scylla-examples)
  - [Create Client](#create-client-1)
  - [Define Model](#define-model-1)
  - [Get](#get-1)
  - [Insert](#insert)
  - [Update](#update)
  - [Delete](#delete)
  - [Count](#count)
  - [Stream](#stream)
  - [Batch (Insert, Update, Delete)](#batch-insert-update-delete---multiple-queries-at-once)
  - [Benchmarking](#benchmarking)
- [Plans for Future](#plans-for-future)
- [Acknowledgments](#acknowledgments)
- [Contributing](#contributing)
- [License](#license)

## Description

- **Support for Multiple Database Clients**: Effortlessly switch between different database clients by specifying the desired features during the build process.
- **Configurable Connection Settings**: Easily define and manage connection parameters for each database client.
- **Simplified Operation Execution**: Perform CRUD (Create, Read, Update, Delete) operations with minimal effort.
- **Batch Processing Capabilities**: Execute multiple operations in a single batch to enhance performance.
- **Streaming Data Support**: Efficiently handle large datasets with built-in support for streaming data processing.
- **Intuitive User Interface**: Enjoy a user-friendly interface for executing queries and managing database interactions.

## Features

- **scylla**: enable ScyllaDb (Cassandra) client
- **redis**: enable Redis/Valkey client

Defaults: []

## Installation

To include `grapple_db` in your project, add it to your dependencies. For example:

```toml
# Example for Cargo.toml (Rust)
[dependencies]
grapple_db = { version = "0.2", features = ["scylla", "redis"] }
scylla = { version = "1.2.0" } // For `scylla` feature
```

## Redis examples

Full code example can be found in [`/examples/redis.rs`](/examples/redis.rs)

### Create Client

```rust
// Default
let client = Client::default().await?;

// From url
let client = Client::from_url("redis://127.0.0.1:6379").await?;

// From config
use grapple_db::redis::pool::Config;
let cfg = Config::from_url("redis://127.0.0.1:6379");

let client = Client::connect(&cfg).await?;
```

### Define Model

```rust
// Imports
use grapple_redis_macros::FromRedisValue;
use grapple_db::redis::RedisModel;

// Define struct
#[derive(Debug, Default, serde::Serialize, serde::Deserialize, FromRedisValue)]
pub struct Model {
    a: i32,
    b: i32,
}

// Implement trait
impl RedisModel for Model {
    fn key(&self) -> String {
        // Key for model
        format!("{}.{}", self.a, self.b)
    }
}
```

### Get

```rust
let key = model.key();
let value: Option<Model> = client.get(&key).await?;
```

### Set

```rust
let model = Model::default();
client.set(&model).await?;
```

### Del

```rust
let key = model.key();
client.del(&key).await?;
```

### Other

```rust
let key = model.key();
let val = client.exists(&key).await?;

let val = client.rename(&key).await?;
```

### Not covered methods

```rust
// For not coveded methods use connection and operate over it
client.connection().await?
    .client_setname::<_, String>("my_client_name".to_string()).await?;
```

## Scylla Examples

Full code example can be found in [`/examples/scylla.rs`](/examples/scylla.rs)

### Create Client

```rust
// With default configuring
let client = Client::default()

// With custom configuring
let con_params = ConnectionParams::default();
let client = Client::connect(&con_params).await?;
```

### Define model

Any of the models could be used with the same client

```rust
#[charybdis_model(
    table_name = users,
    partition_keys = [id],
    clustering_keys = [],

    global_secondary_indexes = [name],
    local_secondary_indexes = [],
)]
#[derive(Debug, Clone, Default)]
pub struct User {
    id: Uuid,
    name: Option<Text>,

    pwd: Option<Text>,
}
```

### Get

```rust
// Any find method that returns one entity
let query = User::find_<any>()
let result = client.get(query).await?;
```

### Insert

```rust
let model = User::default();
client.insert(&model).await?;
```

### Update

```rust
let model = User::default();
client.update(&model).await?;
```

### Delete

```rust
let model = User::default();
client.delete(&model).await?;
```

### Count

```rust
// Any find method that returns stream
let query = User::find_<any>()
let count = client.count(query).await?;
```

### Stream

```rust
// Any find method that returns stream
let query = User::find_<any>()
let stream = client.stream(query).await?;

// Iterate as you want
// There are also available pagable stream
let per_page = 3;
let mut pagable_stream = PagableCharybdisStream::new(stream, per_page);
let mut entities_count = 0;

while let Some(entities) = pagable_stream.next_page().await {
    for entity in entities {
        // Do something with entity
        println!("{:?}", entity);
    }

    entities_count += &entities.len();
}
```

### Batch (Insert, Update, Delete) - multiple queries at once

```rust
let chunk_size = 300;
let entities: Vec<User> = vec![];

client.insert_many(&entities, chunk_size).await?;
client.update_many(&entities, chunk_size).await?;
client.delete_many(&entities, chunk_size).await?;
```

### Benchmarking

I utilized the original benchmarks from the Carybdis ORM, which compared the Scylla Driver with its own performance, and incorporated code from my library. The time difference is minimal and using my library is easier for my use case.

You can check the performance difference by running:

```bash
cargo bench --features scylla --bench scylla_bench
```

Benchmark code available at [`/benches/scylla_bench.rs`](/benches/scylla_bench.rs)

Benchmark results available at [`/benches/scylla_bench.md`](/benches/scylla_bench.md)

## Plans for future

- Implement client for Postgres DB
- Implement client for Mongo DB

## Acknowledgments

The **`scylla`** feature is built on top of the Charybdis ORM, which provides essential functionality for ScyllaDB interactions. For more details on its features and usage, you can find in repo: [`Git`](https://github.com/nodecosmos/charybdis)

## Contributing

Contributions are welcome! If you have suggestions for improvements or new features, feel free to open an issue or submit a pull request. I wrote this library for my own use, so it may not fit everyone's needs, but your input is appreciated!

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
