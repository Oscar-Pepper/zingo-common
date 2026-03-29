//! Transparent address operations for the [`Indexer`] trait.
//!
//! These methods expose transparent (t-address) balance queries, transaction
//! history, and UTXO lookups. They are gated behind the
//! `globally-public-transparent` feature because transparent data is
//! publicly visible on-chain and using these methods leaks which
//! addresses belong to the caller.

use std::future::Future;

use lightwallet_protocol::{
    Address, AddressList, Balance, GetAddressUtxosArg, GetAddressUtxosReply,
    GetAddressUtxosReplyList, RawTransaction, TransparentAddressBlockFilter,
};

pub use super::error::transparent::*;
use super::{GrpcIndexer, Indexer};

/// Extension of [`Indexer`] for transparent address operations.
///
/// All methods in this trait expose globally-public transparent data.
/// Callers can depend on the following:
///
/// - Balance and UTXO results reflect only confirmed (mined) state.
/// - Streaming results are sorted by block height.
/// - Errors are partitioned per method so callers can distinguish
///   which operation failed.
pub trait TransparentIndexer: Indexer {
    type GetTaddressTxidsError: std::error::Error;
    type GetTaddressTransactionsError: std::error::Error;
    type GetTaddressBalanceError: std::error::Error;
    type GetTaddressBalanceStreamError: std::error::Error;
    type GetAddressUtxosError: std::error::Error;
    type GetAddressUtxosStreamError: std::error::Error;

    /// Return a stream of transactions for a transparent address in a block range.
    ///
    /// Same behavior as
    /// [`get_taddress_transactions`](TransparentIndexer::get_taddress_transactions).
    /// This method is a legacy alias; callers should migrate.
    #[deprecated(note = "use get_taddress_transactions instead")]
    fn get_taddress_txids(
        &self,
        filter: TransparentAddressBlockFilter,
    ) -> impl Future<Output = Result<tonic::Streaming<RawTransaction>, Self::GetTaddressTxidsError>>;

    /// Return a stream of transactions for a transparent address in a block range.
    ///
    /// Results are sorted by block height. Mempool transactions are not included.
    fn get_taddress_transactions(
        &self,
        filter: TransparentAddressBlockFilter,
    ) -> impl Future<Output = Result<tonic::Streaming<RawTransaction>, Self::GetTaddressTransactionsError>>;

    /// Return the total confirmed balance for the given transparent addresses.
    ///
    /// The returned [`Balance`] contains the sum in zatoshis. Only confirmed
    /// (mined) outputs are included; mempool UTXOs are not counted.
    fn get_taddress_balance(
        &self,
        addresses: AddressList,
    ) -> impl Future<Output = Result<Balance, Self::GetTaddressBalanceError>>;

    /// Return the total confirmed balance by streaming addresses to the server.
    ///
    /// Client-streaming variant of
    /// [`get_taddress_balance`](TransparentIndexer::get_taddress_balance).
    /// The addresses are streamed individually, avoiding message size limits
    /// for large address sets. Returns the same [`Balance`] sum.
    fn get_taddress_balance_stream(
        &self,
        addresses: Vec<Address>,
    ) -> impl Future<Output = Result<Balance, Self::GetTaddressBalanceStreamError>>;

    /// Return UTXOs for the given addresses as a single response.
    ///
    /// Results are sorted by block height. Pass `max_entries = 0` for
    /// unlimited results.
    fn get_address_utxos(
        &self,
        arg: GetAddressUtxosArg,
    ) -> impl Future<Output = Result<GetAddressUtxosReplyList, Self::GetAddressUtxosError>>;

    /// Return a stream of UTXOs for the given addresses.
    ///
    /// Prefer this over [`get_address_utxos`](TransparentIndexer::get_address_utxos)
    /// when the result set may be large.
    fn get_address_utxos_stream(
        &self,
        arg: GetAddressUtxosArg,
    ) -> impl Future<
        Output = Result<tonic::Streaming<GetAddressUtxosReply>, Self::GetAddressUtxosStreamError>,
    >;
}

impl TransparentIndexer for GrpcIndexer {
    type GetTaddressTxidsError = GetTaddressTxidsError;
    type GetTaddressTransactionsError = GetTaddressTransactionsError;
    type GetTaddressBalanceError = GetTaddressBalanceError;
    type GetTaddressBalanceStreamError = GetTaddressBalanceStreamError;
    type GetAddressUtxosError = GetAddressUtxosError;
    type GetAddressUtxosStreamError = GetAddressUtxosStreamError;

    #[allow(deprecated)]
    async fn get_taddress_txids(
        &self,
        filter: TransparentAddressBlockFilter,
    ) -> Result<tonic::Streaming<RawTransaction>, GetTaddressTxidsError> {
        let (mut client, request) = self.stream_call(filter).await?;
        Ok(client.get_taddress_txids(request).await?.into_inner())
    }

    async fn get_taddress_transactions(
        &self,
        filter: TransparentAddressBlockFilter,
    ) -> Result<tonic::Streaming<RawTransaction>, GetTaddressTransactionsError> {
        let (mut client, request) = self.stream_call(filter).await?;
        Ok(client
            .get_taddress_transactions(request)
            .await?
            .into_inner())
    }

    async fn get_taddress_balance(
        &self,
        addresses: AddressList,
    ) -> Result<Balance, GetTaddressBalanceError> {
        let (mut client, request) = self.time_boxed_call(addresses).await?;
        Ok(client.get_taddress_balance(request).await?.into_inner())
    }

    async fn get_taddress_balance_stream(
        &self,
        addresses: Vec<Address>,
    ) -> Result<Balance, GetTaddressBalanceStreamError> {
        let mut client = self.get_client().await?;
        let stream = tokio_stream::iter(addresses);
        Ok(client
            .get_taddress_balance_stream(stream)
            .await?
            .into_inner())
    }

    async fn get_address_utxos(
        &self,
        arg: GetAddressUtxosArg,
    ) -> Result<GetAddressUtxosReplyList, GetAddressUtxosError> {
        let (mut client, request) = self.time_boxed_call(arg).await?;
        Ok(client.get_address_utxos(request).await?.into_inner())
    }

    async fn get_address_utxos_stream(
        &self,
        arg: GetAddressUtxosArg,
    ) -> Result<tonic::Streaming<GetAddressUtxosReply>, GetAddressUtxosStreamError> {
        let (mut client, request) = self.stream_call(arg).await?;
        Ok(client.get_address_utxos_stream(request).await?.into_inner())
    }
}
