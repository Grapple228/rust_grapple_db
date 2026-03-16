# Changelog for grapple_db

## [0.5.0] - 16 March 2026

### ⚠️ Breaking Changes

- **RedisModel trait**: Added associated type `Value` and required methods `key_ref()` and `value_ref()` for zero-copy operations
- **RedisModelCollector removed**: Replaced with simpler `AsRedisPairs` trait
- **Redis client methods**: Now use `RedisRead` trait instead of `FromRedisValue` for read operations
- **mset/mset_nx**: Now accept arrays of references `[&tuple, &tuple]` instead of slices

### Added

- **RedisRead trait**: New trait for types that can be read from Redis (automatically implemented for types with `FromRedisValue + DeserializeOwned`)
- **AsRedisPairs trait**: Simplified trait for converting collections to key-value pairs
- **RedisPairRef struct**: Wrapper for explicit reference pairs when needed
- **UUID support**: Added to dev-dependencies for unique test keys
- **Binary data examples**: Show how to work with `[u8; N]` and references

### Changed

- **Single implementation for tuples**: One generic `(K, V)` implementation works for all types where `&V: ToRedisArgs`
- **All get methods**: Now require `V: RedisRead` instead of `V: FromRedisValue`
- **Examples**: Updated to show tuple-based approach with manual serialization
- **Tests**: Now use unique keys with UUID to prevent interference
- **Module visibility**: `collector` module is now public

### Fixed

- **Doctests**: All examples now compile and run correctly
- **Type safety**: Better separation between write-only (`RedisModel`) and read/write (`RedisRead`) types
- **Zero-copy**: Operations now work directly with references without unnecessary allocations

### Removed

- **RedisModelCollector trait**: Replaced with simpler `AsRedisPairs`
- **Unused imports**: Cleaned up deadpool-redis imports

## [0.4.1] - 11 March 2026

This version introduces several improvements:

- Replace AsRef<str> with ToRedisArgs for all key parameters
- Add associated type Key to RedisModel trait
- Remove allocation overhead in batch operations
- Support binary keys (Code, State, Nonce) out of the box
- Update examples and tests
- Make redis default feature"

## [0.3.1] - 11 June 2025

This version introduces several improvements:

- Fixes tests and documentation issues.
- Adds a `get_many` method to the Scylla client for retrieving multiple entities efficiently.
- Modifies the `del` method in the Redis client to return a boolean indicating successful deletion.

## [0.3.0] - 11 June 2025

This release improves using of `redis` feature. There are some breaking changes

- Updates to `grapple_redis_macros` to version 0.2.0.
- Adds `batch` methods to Redis client for `mget`, `mset`, and `mdel` operations.
- Improves documentation and examples.
- Includes `futures` dependency for Redis feature.
- Added tests for `scylla` client
- Added tests for `redis` client

## [0.2.1] - 08 June 2025

### Fixes

- `FromRedisValue` of `grapple_redis_macros` moved into `grapple_db/redis/macros` and available without external dependensies
- Documentation Fix because of macro location changed.

## [0.2.0] - 08 June 2025

This release contains some breaking changes. The `Charybdis` library has been integrated into `grapple_db`, allowing it to be used directly from there. As a result, the module structure has been changed.

### Added

- Introduced a new `redis` feature to enhance functionality.
- Added a Redis `Client` for improved interaction with Redis databases.
- Implemented all basic operations for the `Redis` client.
- Integrated a connection pool for efficient management of `Redis` connections.
- Added `redis-macro` for easier macro-based implementations.
- Included comprehensive tests for the Redis client to ensure reliability and correctness.
- Implemented `serde::Serialize` for `scylla::Error` and `redis::Error` to facilitate error serialization in JSON format.
- Covered all code with comments and examples.

### Removed

- Removed unused libraries to streamline the project.
- Default features set to [].

### Fixes

- Updated documentation for the Scylla `Client`, ensuring that doctests run without errors.
- `charybdis` are now available from the library without dependency.

## [0.1.2] - 03 June 2025

### Fixes

- Removed drop_and_replace in migrate

## [0.1.1] - 03 June 2025

### Fixes

- Fixed mistakes in comments

## [0.1.0] - 03 June 2025

### Added

- **Feature 'scylla'**: enables the Scylla DB client.
- **Stream functionality**: added support for streamed data processing.
- **CRUD functionality**: implemented create, read, update, count and delete operations.
- **Batch functionality**: allows for batch processing of operations.
- **Params**: added `ConnectionParams` and `CrudParams` , defining options for creating connection and query execution.
- **Comments**: added comments with examples.
