# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `Indexer` trait defining the interface for communicating with a Zcash chain indexer,
  with methods: `get_info`, `get_latest_block`, `send_transaction`, `get_trees`.
- `GrpcIndexer` struct implementing `Indexer` over gRPC to a lightwalletd server.
  Holds an `Option<http::Uri>` and provides `new`, `disconnected`, `uri`, `set_uri`,
  `disconnect`, and `get_client` (inherent) methods.
- `GrpcIndexerError` error type covering `GetClientError`, `tonic::Status`, and
  server-side send rejections.
- `GetClientError::NoUri` variant for when `GrpcIndexer` has no URI set.

### Changed

- Support for Zebra 4.1.0 through `zebra-chain = "5.0"`
- Bump `tonic` from `0.13` to `0.14`, with `tls-webpki-roots` enabled.
- **Breaking:** Replace free function `get_client(uri)` with `GrpcIndexer::get_client(&self)`.
  Callers must change `get_client(uri).await` to `GrpcIndexer::new(uri).get_client().await`.
- **Breaking:** `get_client` now returns `CompactTxStreamerClient<Channel>` instead of
  `CompactTxStreamerClient<UnderlyingService>`. TLS and transport are handled internally
  by tonic.
- **Breaking:** `GetClientError` gains `Transport` (wrapping `tonic::transport::Error`)
  and `NoUri` variants.

### Removed

- `client` module and `client_from_connector` utility function.
- `http-body` dependency.
- `GrpcConnector` struct, `GrpcConnector::new()`, and `GrpcConnector::uri()`.
- `UnderlyingService` type alias (`BoxCloneService<...>`).
- Free function `get_client(uri: http::Uri)` — replaced by `GrpcIndexer::get_client`.
- Manual URI rewrite logic (scheme/authority injection into requests); now handled
  internally by tonic's `Endpoint`.
- Direct dependencies on `tower` and `webpki-roots` (TLS root certs now provided by
  tonic's `tls-webpki-roots` feature).

## [1.1.0]

NOT PUBLISHED
