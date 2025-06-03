# grapple_db

`grapple_db` is a library designed to simplify interactions with various databases. It offers a flexible architecture that allows you to select the database client you need and utilize it according to your requirements. The library can be easily integrated into your project using feature flags.

## Descreption

- **Support for Multiple Database Clients**: Effortlessly switch between different database clients by specifying the desired features during the build process.
- **Configurable Connection Settings**: Easily define and manage connection parameters for each database client.
- **Simplified Operation Execution**: Perform CRUD (Create, Read, Update, Delete) operations with minimal effort.
- **Batch Processing Capabilities**: Execute multiple operations in a single batch to enhance performance.
- **Streaming Data Support**: Efficiently handle large datasets with built-in support for streaming data processing.
- **Intuitive User Interface**: Enjoy a user-friendly interface for executing queries and managing database interactions.

## Features

- **scylla**: enable ScyllaDb (Cassandra) client

## Installation

To include `grapple_db` in your project, add it to your dependencies. If you are using a package manager, you can specify the features you need. For example:

```toml
# Example for Cargo.toml (Rust)
[dependencies]
grapple_db = { version = "0.1.0", features = ["scylla"] }
charybdis = { version = "1.0.1", features = ["migrate"] }
scylla = { version = "1.2.0" }
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
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
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

## Benchmarking

I utilized the original benchmarks from the Carybdis ORM, which compared the Scylla Driver with its own performance, and incorporated code from my library. The time difference is minimal and using my library is easier for my use case.

You can check the performance difference by running:

```bash
cargo bench --features scylla --bench scylla_bench
```

Benchmark code available at [`/benches/scylla_bench.rs`](/benches/scylla_bench.rs)

Benchmark results available at [`/benches/scylla_bench.md`](/benches/scylla_bench.md)

## Plans for future

- Add tests for modules
- Implement client for Redis/Valkey DB
- Implement client for Postgres DB
- Implement client for Mongo DB

## Acknowledgments

The **`scylla`** feature is built on top of the Charybdis ORM, which provides essential functionality for ScyllaDB interactions. For more details on its features and usage, you can find in repo: [`Git`](https://github.com/nodecosmos/charybdis)

## Contributing

Contributions are welcome! If you have suggestions for improvements or new features, feel free to open an issue or submit a pull request. I wrote this library for my own use, so it may not fit everyone's needs, but your input is appreciated!

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
