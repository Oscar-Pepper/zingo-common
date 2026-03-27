# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [4.0.0]

### Added

- `Indexer` trait covering the full `CompactTxStreamer` gRPC service:
  `get_info`, `get_latest_block`, `send_transaction`, `get_tree_state`,
  `get_block`, `get_block_range`, `get_transaction`, `get_mempool_tx`,
  `get_mempool_stream`, `get_latest_tree_state`, `get_subtree_roots`.
- Per-method associated error types on the `Indexer` trait.
- `RpcError` common error type for methods with no additional failure modes.
- `GrpcIndexer` struct implementing `Indexer` over gRPC. Validates URI at
  construction (`new` returns `Result`) and pre-builds the TLS endpoint.
- `get_client` inherent method on `GrpcIndexer` returning
  `CompactTxStreamerClient<Channel>` from `lightwallet_protocol`.
- `pub use lightwallet_protocol` re-export so consumers can access proto
  types via `zingo_netutils::lightwallet_protocol::*`.
- `globally-public-transparent` feature gate (off by default) for
  transparent address methods: `get_taddress_txids` (deprecated),
  `get_taddress_transactions`, `get_taddress_balance`,
  `get_taddress_balance_stream`, `get_address_utxos`,
  `get_address_utxos_stream`.
- `back_compatible` feature gate (off by default) providing
  `get_zcb_client()` which returns `zcash_client_backend`'s
  `CompactTxStreamerClient<Channel>` for pepper-sync compatibility.
- Deprecated trait methods: `get_block_nullifiers`, `get_block_range_nullifiers`,
  `get_taddress_txids`.

### Changed

- **Breaking:** Replace `zcash_client_backend` with `lightwallet-protocol`
  for all proto-generated types. Consumers must update imports.
- **Breaking:** `GrpcIndexer::new(uri)` now returns `Result<Self, GetClientError>`
  (validates scheme and authority at construction).
- **Breaking:** `uri()` returns `&http::Uri` (not `Option`).
- **Breaking:** Per-method error types (`GetInfoError`, `GetLatestBlockError`,
  `SendTransactionError`, `GetTreesError`) replace the single `GrpcIndexerError`.
- **Breaking:** Renamed `get_trees` to `get_tree_state`.
- Bump `tonic` to `0.14`, `lightwallet-protocol` to `0.3`.
- `hyper`, `hyper-rustls`, `hyper-util` moved from dependencies to dev-dependencies.

### Removed

- `zcash_client_backend` dependency (available optionally via `back_compatible`).
- `set_uri`, `disconnect`, `disconnected` methods.
- `GrpcIndexerError` unified error type.
- `GetClientError::NoUri` variant.
- `Option<http::Uri>` internal state — `GrpcIndexer` always holds a valid URI.
- `client` module, `GrpcConnector`, `UnderlyingService`, free `get_client` function.
- Direct dependencies on `tower`, `webpki-roots`, `zebra-chain`.

## [1.1.0]

NOT PUBLISHED
