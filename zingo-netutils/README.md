# zingo-netutils

A complete `Indexer` abstraction for communicating with Zcash chain indexers
(`lightwalletd` / `zainod`).

## Organizing principle

This crate defines the **`Indexer` trait** -- the sole interface a Zcash
wallet or tool needs to query, sync, and broadcast against a chain indexer.
The trait is implementation-agnostic: production code uses the provided
`GrpcIndexer` (gRPC over tonic), while tests can supply a mock implementor
with no network dependency.

All proto types are provided by
[`lightwallet-protocol`](https://crates.io/crates/lightwallet-protocol)
and re-exported via `pub use lightwallet_protocol` so consumers do not need
an additional dependency.

## Usage

```rust
use zingo_netutils::{GrpcIndexer, Indexer};

let uri: http::Uri = "https://zec.rocks:443".parse().unwrap();
let indexer = GrpcIndexer::new(uri)?;

let info = indexer.get_info().await?;
let tip  = indexer.get_latest_block().await?;
```

`GrpcIndexer::new` validates the URI (scheme must be `http` or `https`,
authority must be present) and pre-builds the TLS endpoint. It returns
`Result<Self, GetClientError>` -- there is no invalid-state representation.

## Backwards compatibility

Code that needs a raw `CompactTxStreamerClient<Channel>` (e.g. pepper-sync)
can still obtain one:

```rust
// Using lightwallet-protocol types (preferred)
let client = indexer.get_client().await?;

// Using zcash_client_backend types (migration aid)
// Requires: zingo-netutils = { features = ["back_compatible"] }
let client = indexer.get_zcb_client().await?;
```

The `back_compatible` feature brings in `zcash_client_backend` as an
optional dependency and exposes `get_zcb_client()`, which returns
`zcash_client_backend`'s `CompactTxStreamerClient<Channel>`. This is a
bridge for consumers that have not yet migrated to `lightwallet-protocol`
types.

## Feature gates

All features are **off by default**.

| Feature | What it enables |
|---|---|
| `globally-public-transparent` | `TransparentIndexer` sub-trait: t-address balance, transaction history, and UTXO queries. Requires `tokio-stream`. |
| `ping-very-insecure` | `Indexer::ping()` method. Name mirrors the lightwalletd `--ping-very-insecure` CLI flag required server-side. Testing only. |
| `back_compatible` | `GrpcIndexer::get_zcb_client()` returning `zcash_client_backend`'s client type for pepper-sync compatibility. |

## Error handling

Every trait method has a dedicated error enum (defined in `src/error.rs`).
Each enum has two variants:

- `GetClientError(GetClientError)` — the connection was never established.
  `InvalidScheme` and `InvalidAuthority` are deterministic; `Transport`
  may be transient and retryable.
- A method-specific gRPC variant wrapping `tonic::Status` — the server
  received the request but returned an error.

`SendTransactionError` adds a third variant, `SendRejected(String)`, for
transactions the server evaluated and rejected (not retryable with the
same bytes).

All error types are bounded by `std::error::Error`. Transparent method
errors live in `error::transparent` (gated by `globally-public-transparent`).

| Error type | Used by |
|---|---|
| `GetClientError` | `GrpcIndexer::new`, `get_client`, embedded in every method error |
| `GetInfoError` | `get_info` |
| `GetLatestBlockError` | `get_latest_block` |
| `SendTransactionError` | `send_transaction` |
| `GetTreeStateError` | `get_tree_state` |
| `GetBlockError` | `get_block` |
| `GetBlockNullifiersError` | `get_block_nullifiers` |
| `GetBlockRangeError` | `get_block_range` |
| `GetBlockRangeNullifiersError` | `get_block_range_nullifiers` |
| `GetTransactionError` | `get_transaction` |
| `GetMempoolTxError` | `get_mempool_tx` |
| `GetMempoolStreamError` | `get_mempool_stream` |
| `GetLatestTreeStateError` | `get_latest_tree_state` |
| `GetSubtreeRootsError` | `get_subtree_roots` |
| `PingError` | `ping` |
| `GetTaddressTxidsError` | `get_taddress_txids` |
| `GetTaddressTransactionsError` | `get_taddress_transactions` |
| `GetTaddressBalanceError` | `get_taddress_balance` |
| `GetTaddressBalanceStreamError` | `get_taddress_balance_stream` |
| `GetAddressUtxosError` | `get_address_utxos` |
| `GetAddressUtxosStreamError` | `get_address_utxos_stream` |

## TLS

HTTPS connections use rustls with webpki root certificates (via tonic's
`tls-webpki-roots` feature). No system OpenSSL installation is required.

## gRPC endpoint audit

Every `CompactTxStreamer` RPC from `walletrpc/service.proto` is covered.
The table below shows the mapping between proto RPCs and trait methods.

### Exact matches (13)

| Proto RPC | Trait method | Input | Output |
|---|---|---|---|
| `GetTreeState(BlockID)` | `get_tree_state(BlockId)` | `BlockId` | `TreeState` |
| `GetBlock(BlockID)` | `get_block(BlockId)` | `BlockId` | `CompactBlock` |
| `GetBlockNullifiers(BlockID)` | `get_block_nullifiers(BlockId)` | `BlockId` | `CompactBlock` |
| `GetBlockRange(BlockRange)` | `get_block_range(BlockRange)` | `BlockRange` | `Streaming<CompactBlock>` |
| `GetBlockRangeNullifiers(BlockRange)` | `get_block_range_nullifiers(BlockRange)` | `BlockRange` | `Streaming<CompactBlock>` |
| `GetTransaction(TxFilter)` | `get_transaction(TxFilter)` | `TxFilter` | `RawTransaction` |
| `GetTaddressTxids(TABF)` | `get_taddress_txids(TABF)` | `TABF` | `Streaming<RawTransaction>` |
| `GetTaddressTransactions(TABF)` | `get_taddress_transactions(TABF)` | `TABF` | `Streaming<RawTransaction>` |
| `GetTaddressBalance(AddressList)` | `get_taddress_balance(AddressList)` | `AddressList` | `Balance` |
| `GetMempoolTx(GetMempoolTxRequest)` | `get_mempool_tx(GetMempoolTxRequest)` | `GetMempoolTxRequest` | `Streaming<CompactTx>` |
| `GetSubtreeRoots(GetSubtreeRootsArg)` | `get_subtree_roots(GetSubtreeRootsArg)` | `GetSubtreeRootsArg` | `Streaming<SubtreeRoot>` |
| `GetAddressUtxos(GetAddressUtxosArg)` | `get_address_utxos(GetAddressUtxosArg)` | `GetAddressUtxosArg` | `GetAddressUtxosReplyList` |
| `GetAddressUtxosStream(GetAddressUtxosArg)` | `get_address_utxos_stream(GetAddressUtxosArg)` | `GetAddressUtxosArg` | `Streaming<GetAddressUtxosReply>` |
| `Ping(Duration)` | `ping(ProtoDuration)` | `ProtoDuration` | `PingResponse` |

### Intentional divergences (7)

| Proto RPC | Trait method | What diverges | Why |
|---|---|---|---|
| `GetLightdInfo(Empty)` | `get_info()` | Hides `Empty` arg | Ergonomic -- unit request type adds no information |
| `GetLatestBlock(ChainSpec)` | `get_latest_block()` | Hides `ChainSpec` arg | Same reason |
| `GetMempoolStream(Empty)` | `get_mempool_stream()` | Hides `Empty` arg | Same reason |
| `GetLatestTreeState(Empty)` | `get_latest_tree_state()` | Hides `Empty` arg | Same reason |
| `SendTransaction(RawTransaction)` | `send_transaction(Box<[u8]>)` | Abstracts input and output | Takes raw bytes, returns txid string -- decouples callers from proto |
| `GetTaddressBalanceStream(stream Address)` | `get_taddress_balance_stream(Vec<Address>)` | `Vec` instead of client stream | Impl converts to stream internally; simpler call-site API |
| `Ping(Duration)` | `ping(ProtoDuration)` | Type alias | Avoids collision with `std::time::Duration` |

### Compile-time enforcement

The `proto_agreement` test module (`src/proto_agreement.rs`) contains 20
dead-code async functions that reference both the generated
`CompactTxStreamerClient` method and the corresponding trait method with
explicit type annotations. If either side drifts from the proto, the
module fails to compile.

## License

[MIT](LICENSE-MIT)
