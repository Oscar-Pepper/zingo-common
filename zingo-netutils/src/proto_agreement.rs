//! Compile-time assertions that each trait method maps to the correct
//! [`CompactTxStreamerClient`] method with compatible types.
//!
//! The proto source of truth is `walletrpc/service.proto` from the
//! [`lightwallet-protocol`](https://crates.io/crates/lightwallet-protocol)
//! crate (see `Cargo.toml` for the pinned version). That crate compiles
//! `service.proto` and `compact_formats.proto` from
//! <https://github.com/zingolabs/lightwallet-protocol-rust> at build time.
//!
//! These functions are never called — they exist only to fail compilation
//! if the proto-generated client and the trait diverge. Each function
//! documents the proto RPC it covers.

#![allow(dead_code, unused_variables, clippy::needless_return)]

use super::*;

// ── Indexer trait ────────────────────────────────────────────────────

// Proto: GetLightdInfo(Empty) -> LightdInfo
// Trait: get_info() -> LightdInfo  (Empty hidden by impl)
async fn get_lightd_info(c: &mut CompactTxStreamerClient<Channel>, i: &mut GrpcIndexer) {
    let _p: LightdInfo = c
        .get_lightd_info(Request::new(Empty {}))
        .await
        .unwrap()
        .into_inner();
    let _t: LightdInfo = i.get_lightd_info().await.unwrap();
}

// Proto: GetLatestBlock(ChainSpec) -> BlockID
// Trait: get_latest_block() -> BlockId  (ChainSpec hidden by impl)
async fn get_latest_block(c: &mut CompactTxStreamerClient<Channel>, i: &mut GrpcIndexer) {
    let _p: BlockId = c
        .get_latest_block(Request::new(ChainSpec {}))
        .await
        .unwrap()
        .into_inner();
    let _t: BlockId = i.get_latest_block().await.unwrap();
}

// Proto: SendTransaction(RawTransaction) -> SendResponse
// Trait: send_transaction(Box<[u8]>) -> String  (abstracts both sides)
async fn send_transaction(c: &mut CompactTxStreamerClient<Channel>, i: &mut GrpcIndexer) {
    let _p: lightwallet_protocol::SendResponse = c
        .send_transaction(Request::new(RawTransaction {
            data: vec![],
            height: 0,
        }))
        .await
        .unwrap()
        .into_inner();
    let _t: String = i.send_transaction(vec![].into_boxed_slice()).await.unwrap();
}

// Proto: GetTreeState(BlockID) -> TreeState
// Trait: get_tree_state(BlockId) -> TreeState
async fn get_tree_state(c: &mut CompactTxStreamerClient<Channel>, i: &mut GrpcIndexer) {
    let id = BlockId {
        height: 0,
        hash: vec![],
    };
    let _p: TreeState = c
        .get_tree_state(Request::new(id.clone()))
        .await
        .unwrap()
        .into_inner();
    let _t: TreeState = i.get_tree_state(id).await.unwrap();
}

// Proto: GetBlock(BlockID) -> CompactBlock
// Trait: get_block(BlockId) -> CompactBlock
async fn get_block(c: &mut CompactTxStreamerClient<Channel>, i: &mut GrpcIndexer) {
    let id = BlockId {
        height: 0,
        hash: vec![],
    };
    let _p: CompactBlock = c
        .get_block(Request::new(id.clone()))
        .await
        .unwrap()
        .into_inner();
    let _t: CompactBlock = i.get_block(id).await.unwrap();
}

// Proto: GetBlockNullifiers(BlockID) -> CompactBlock  [deprecated]
// Trait: get_block_nullifiers(BlockId) -> CompactBlock
#[allow(deprecated)]
async fn get_block_nullifiers(c: &mut CompactTxStreamerClient<Channel>, i: &mut GrpcIndexer) {
    let id = BlockId {
        height: 0,
        hash: vec![],
    };
    let _p: CompactBlock = c
        .get_block_nullifiers(Request::new(id.clone()))
        .await
        .unwrap()
        .into_inner();
    let _t: CompactBlock = i.get_block_nullifiers(id).await.unwrap();
}

// Proto: GetBlockRange(BlockRange) -> stream CompactBlock
// Trait: get_block_range(BlockRange) -> Streaming<CompactBlock>
async fn get_block_range(c: &mut CompactTxStreamerClient<Channel>, i: &mut GrpcIndexer) {
    let range = BlockRange {
        start: None,
        end: None,
        pool_types: vec![],
    };
    let _p: tonic::Streaming<CompactBlock> = c
        .get_block_range(Request::new(range.clone()))
        .await
        .unwrap()
        .into_inner();
    let _t: tonic::Streaming<CompactBlock> = i.get_block_range(range).await.unwrap();
}

// Proto: GetBlockRangeNullifiers(BlockRange) -> stream CompactBlock  [deprecated]
// Trait: get_block_range_nullifiers(BlockRange) -> Streaming<CompactBlock>
#[allow(deprecated)]
async fn get_block_range_nullifiers(c: &mut CompactTxStreamerClient<Channel>, i: &mut GrpcIndexer) {
    let range = BlockRange {
        start: None,
        end: None,
        pool_types: vec![],
    };
    let _p: tonic::Streaming<CompactBlock> = c
        .get_block_range_nullifiers(Request::new(range.clone()))
        .await
        .unwrap()
        .into_inner();
    let _t: tonic::Streaming<CompactBlock> = i.get_block_range_nullifiers(range).await.unwrap();
}

// Proto: GetTransaction(TxFilter) -> RawTransaction
// Trait: get_transaction(TxFilter) -> RawTransaction
async fn get_transaction(c: &mut CompactTxStreamerClient<Channel>, i: &mut GrpcIndexer) {
    let filter = TxFilter {
        block: None,
        index: 0,
        hash: vec![],
    };
    let _p: RawTransaction = c
        .get_transaction(Request::new(filter.clone()))
        .await
        .unwrap()
        .into_inner();
    let _t: RawTransaction = i.get_transaction(filter).await.unwrap();
}

// Proto: GetMempoolTx(GetMempoolTxRequest) -> stream CompactTx
// Trait: get_mempool_tx(GetMempoolTxRequest) -> Streaming<CompactTx>
async fn get_mempool_tx(c: &mut CompactTxStreamerClient<Channel>, i: &mut GrpcIndexer) {
    let req = GetMempoolTxRequest {
        exclude_txid_suffixes: vec![],
        pool_types: vec![],
    };
    let _p: tonic::Streaming<CompactTx> = c
        .get_mempool_tx(Request::new(req.clone()))
        .await
        .unwrap()
        .into_inner();
    let _t: tonic::Streaming<CompactTx> = i.get_mempool_tx(req).await.unwrap();
}

// Proto: GetMempoolStream(Empty) -> stream RawTransaction
// Trait: get_mempool_stream() -> Streaming<RawTransaction>  (Empty hidden by impl)
async fn get_mempool_stream(c: &mut CompactTxStreamerClient<Channel>, i: &mut GrpcIndexer) {
    let _p: tonic::Streaming<RawTransaction> = c
        .get_mempool_stream(Request::new(Empty {}))
        .await
        .unwrap()
        .into_inner();
    let _t: tonic::Streaming<RawTransaction> = i.get_mempool_stream().await.unwrap();
}

// Proto: GetLatestTreeState(Empty) -> TreeState
// Trait: get_latest_tree_state() -> TreeState  (Empty hidden by impl)
async fn get_latest_tree_state(c: &mut CompactTxStreamerClient<Channel>, i: &mut GrpcIndexer) {
    let _p: TreeState = c
        .get_latest_tree_state(Request::new(Empty {}))
        .await
        .unwrap()
        .into_inner();
    let _t: TreeState = i.get_latest_tree_state().await.unwrap();
}

// Proto: GetSubtreeRoots(GetSubtreeRootsArg) -> stream SubtreeRoot
// Trait: get_subtree_roots(GetSubtreeRootsArg) -> Streaming<SubtreeRoot>
async fn get_subtree_roots(c: &mut CompactTxStreamerClient<Channel>, i: &mut GrpcIndexer) {
    let arg = GetSubtreeRootsArg {
        start_index: 0,
        shielded_protocol: 0,
        max_entries: 0,
    };
    let _p: tonic::Streaming<SubtreeRoot> = c
        .get_subtree_roots(Request::new(arg))
        .await
        .unwrap()
        .into_inner();
    let _t: tonic::Streaming<SubtreeRoot> = i.get_subtree_roots(arg).await.unwrap();
}

// Proto: Ping(Duration) -> PingResponse
// Trait: ping(ProtoDuration) -> PingResponse  (aliased to avoid std collision)
#[cfg(feature = "ping-very-insecure")]
async fn ping(c: &mut CompactTxStreamerClient<Channel>, i: &mut GrpcIndexer) {
    let dur = ProtoDuration { interval_us: 0 };
    let _p: PingResponse = c.ping(Request::new(dur)).await.unwrap().into_inner();
    let _t: PingResponse = i.ping(dur).await.unwrap();
}

// ── TransparentIndexer trait ────────────────────────────────────────

#[cfg(feature = "globally-public-transparent")]
mod transparent {
    use super::*;
    use lightwallet_protocol::{
        Address, AddressList, Balance, GetAddressUtxosArg, GetAddressUtxosReply,
        GetAddressUtxosReplyList, TransparentAddressBlockFilter,
    };

    // Proto: GetTaddressTxids(TransparentAddressBlockFilter) -> stream RawTransaction  [deprecated]
    // Trait: get_taddress_txids(TABF) -> Streaming<RawTransaction>
    #[allow(deprecated)]
    async fn get_taddress_txids(c: &mut CompactTxStreamerClient<Channel>, i: &mut GrpcIndexer) {
        let f = TransparentAddressBlockFilter {
            address: String::new(),
            range: None,
        };
        let _p: tonic::Streaming<RawTransaction> = c
            .get_taddress_txids(Request::new(f.clone()))
            .await
            .unwrap()
            .into_inner();
        let _t: tonic::Streaming<RawTransaction> = i.get_taddress_txids(f).await.unwrap();
    }

    // Proto: GetTaddressTransactions(TransparentAddressBlockFilter) -> stream RawTransaction
    // Trait: get_taddress_transactions(TABF) -> Streaming<RawTransaction>
    async fn get_taddress_transactions(c: &mut CompactTxStreamerClient<Channel>, i: &mut GrpcIndexer) {
        let f = TransparentAddressBlockFilter {
            address: String::new(),
            range: None,
        };
        let _p: tonic::Streaming<RawTransaction> = c
            .get_taddress_transactions(Request::new(f.clone()))
            .await
            .unwrap()
            .into_inner();
        let _t: tonic::Streaming<RawTransaction> = i.get_taddress_transactions(f).await.unwrap();
    }

    // Proto: GetTaddressBalance(AddressList) -> Balance
    // Trait: get_taddress_balance(AddressList) -> Balance
    async fn get_taddress_balance(c: &mut CompactTxStreamerClient<Channel>, i: &mut GrpcIndexer) {
        let addrs = AddressList { addresses: vec![] };
        let _p: Balance = c
            .get_taddress_balance(Request::new(addrs.clone()))
            .await
            .unwrap()
            .into_inner();
        let _t: Balance = i.get_taddress_balance(addrs).await.unwrap();
    }

    // Proto: GetTaddressBalanceStream(stream Address) -> Balance
    // Trait: get_taddress_balance_stream(Vec<Address>) -> Balance  (Vec streamed by impl)
    async fn get_taddress_balance_stream(
        c: &mut CompactTxStreamerClient<Channel>,
        i: &mut GrpcIndexer,
    ) {
        let addrs = vec![Address {
            address: String::new(),
        }];
        let _p: Balance = c
            .get_taddress_balance_stream(tokio_stream::iter(addrs.clone()))
            .await
            .unwrap()
            .into_inner();
        let _t: Balance = i.get_taddress_balance_stream(addrs).await.unwrap();
    }

    // Proto: GetAddressUtxos(GetAddressUtxosArg) -> GetAddressUtxosReplyList
    // Trait: get_address_utxos(GetAddressUtxosArg) -> GetAddressUtxosReplyList
    async fn get_address_utxos(c: &mut CompactTxStreamerClient<Channel>, i: &mut GrpcIndexer) {
        let arg = GetAddressUtxosArg {
            addresses: vec![],
            start_height: 0,
            max_entries: 0,
        };
        let _p: GetAddressUtxosReplyList = c
            .get_address_utxos(Request::new(arg.clone()))
            .await
            .unwrap()
            .into_inner();
        let _t: GetAddressUtxosReplyList = i.get_address_utxos(arg).await.unwrap();
    }

    // Proto: GetAddressUtxosStream(GetAddressUtxosArg) -> stream GetAddressUtxosReply
    // Trait: get_address_utxos_stream(GetAddressUtxosArg) -> Streaming<GetAddressUtxosReply>
    async fn get_address_utxos_stream(c: &mut CompactTxStreamerClient<Channel>, i: &mut GrpcIndexer) {
        let arg = GetAddressUtxosArg {
            addresses: vec![],
            start_height: 0,
            max_entries: 0,
        };
        let _p: tonic::Streaming<GetAddressUtxosReply> = c
            .get_address_utxos_stream(Request::new(arg.clone()))
            .await
            .unwrap()
            .into_inner();
        let _t: tonic::Streaming<GetAddressUtxosReply> =
            i.get_address_utxos_stream(arg).await.unwrap();
    }
}
