//! Error types for the [`Indexer`](super::Indexer) and
//! `TransparentIndexer` traits.
//!
//! Each trait method has a dedicated error enum so callers can
//! distinguish connection failures ([`GetClientError`]) from
//! server-side gRPC errors ([`tonic::Status`]).

/// Connection-level error returned by [`GrpcIndexer::new`](super::GrpcIndexer::new)
/// and the connection phase of every trait method.
///
/// Callers can depend on:
/// - `InvalidScheme` and `InvalidAuthority` are deterministic — retrying
///   with the same URI will always fail.
/// - `Transport` wraps a [`tonic::transport::Error`] and may be transient
///   (e.g. DNS resolution, TCP connect timeout). Retrying may succeed.
#[derive(Debug, thiserror::Error)]
pub enum GetClientError {
    #[error("bad uri: invalid scheme")]
    InvalidScheme,

    #[error("bad uri: invalid authority")]
    InvalidAuthority,

    #[error(transparent)]
    Transport(#[from] tonic::transport::Error),
}

#[cfg(test)]
mod get_client_error_tests {
    use super::*;

    #[test]
    fn invalid_scheme() {
        let e = GetClientError::InvalidScheme;
        assert!(matches!(e, GetClientError::InvalidScheme));
        assert_eq!(e.to_string(), "bad uri: invalid scheme");
    }

    #[test]
    fn invalid_authority() {
        let e = GetClientError::InvalidAuthority;
        assert!(matches!(e, GetClientError::InvalidAuthority));
        assert_eq!(e.to_string(), "bad uri: invalid authority");
    }

    #[test]
    fn transport_from_conversion() {
        // Verify the From impl exists at compile time.
        let _: fn(tonic::transport::Error) -> GetClientError = GetClientError::from;
    }
}

/// Error from the `get_info` (`GetLightdInfo`) RPC.
///
/// Callers can depend on:
/// - `GetClientError` means the connection was never established.
/// - `GetLightdInfoError` means the server received the request but
///   returned a gRPC status (e.g. `Unavailable`, `Internal`).
#[derive(Debug, thiserror::Error)]
pub enum GetInfoError {
    #[error(transparent)]
    GetClientError(#[from] GetClientError),

    #[error("gRPC error: {0}")]
    GetLightdInfoError(#[from] tonic::Status),
}

/// Error from the `get_latest_block` (`GetLatestBlock`) RPC.
///
/// Callers can depend on:
/// - `GetClientError` means the connection was never established.
/// - `GetLatestBlockError` means the server returned a gRPC status.
#[derive(Debug, thiserror::Error)]
pub enum GetLatestBlockError {
    #[error(transparent)]
    GetClientError(#[from] GetClientError),

    #[error("gRPC error: {0}")]
    GetLatestBlockError(#[from] tonic::Status),
}

/// Error from the `send_transaction` (`SendTransaction`) RPC.
///
/// Callers can depend on:
/// - `GetClientError` means the connection was never established.
/// - `SendTransactionError` means the server returned a gRPC status
///   before evaluating the transaction.
/// - `SendRejected` means the server received and evaluated the
///   transaction but rejected it (e.g. duplicate, invalid). The string
///   contains the server's rejection reason. This is **not** retryable
///   with the same transaction bytes.
#[derive(Debug, thiserror::Error)]
pub enum SendTransactionError {
    #[error(transparent)]
    GetClientError(#[from] GetClientError),

    #[error("gRPC error: {0}")]
    SendTransactionError(#[from] tonic::Status),

    #[error("send rejected: {0}")]
    SendRejected(String),
}

/// Error from the `get_tree_state` (`GetTreeState`) RPC.
///
/// Callers can depend on:
/// - `GetClientError` means the connection was never established.
/// - `GetTreeStateError` means the server returned a gRPC status
///   (e.g. the requested block does not exist).
#[derive(Debug, thiserror::Error)]
pub enum GetTreeStateError {
    #[error(transparent)]
    GetClientError(#[from] GetClientError),

    #[error("gRPC error: {0}")]
    GetTreeStateError(#[from] tonic::Status),
}

/// Error from the `get_block` (`GetBlock`) RPC.
///
/// Callers can depend on:
/// - `GetClientError` means the connection was never established.
/// - `GetBlockError` means the server returned a gRPC status
///   (e.g. the requested block does not exist).
#[derive(Debug, thiserror::Error)]
pub enum GetBlockError {
    #[error(transparent)]
    GetClientError(#[from] GetClientError),

    #[error("gRPC error: {0}")]
    GetBlockError(#[from] tonic::Status),
}

/// Error from the deprecated `get_block_nullifiers` (`GetBlockNullifiers`) RPC.
///
/// Callers can depend on:
/// - `GetClientError` means the connection was never established.
/// - `GetBlockNullifiersError` means the server returned a gRPC status.
#[derive(Debug, thiserror::Error)]
pub enum GetBlockNullifiersError {
    #[error(transparent)]
    GetClientError(#[from] GetClientError),

    #[error("gRPC error: {0}")]
    GetBlockNullifiersError(#[from] tonic::Status),
}

/// Error from the `get_block_range` (`GetBlockRange`) RPC.
///
/// Callers can depend on:
/// - `GetClientError` means the connection was never established.
/// - `GetBlockRangeError` means the server returned a gRPC status
///   before or instead of streaming blocks.
#[derive(Debug, thiserror::Error)]
pub enum GetBlockRangeError {
    #[error(transparent)]
    GetClientError(#[from] GetClientError),

    #[error("gRPC error: {0}")]
    GetBlockRangeError(#[from] tonic::Status),
}

/// Error from the deprecated `get_block_range_nullifiers` (`GetBlockRangeNullifiers`) RPC.
///
/// Callers can depend on:
/// - `GetClientError` means the connection was never established.
/// - `GetBlockRangeNullifiersError` means the server returned a gRPC status.
#[derive(Debug, thiserror::Error)]
pub enum GetBlockRangeNullifiersError {
    #[error(transparent)]
    GetClientError(#[from] GetClientError),

    #[error("gRPC error: {0}")]
    GetBlockRangeNullifiersError(#[from] tonic::Status),
}

/// Error from the `get_transaction` (`GetTransaction`) RPC.
///
/// Callers can depend on:
/// - `GetClientError` means the connection was never established.
/// - `GetTransactionError` means the server returned a gRPC status
///   (e.g. the transaction was not found).
#[derive(Debug, thiserror::Error)]
pub enum GetTransactionError {
    #[error(transparent)]
    GetClientError(#[from] GetClientError),

    #[error("gRPC error: {0}")]
    GetTransactionError(#[from] tonic::Status),
}

/// Error from the `get_mempool_tx` (`GetMempoolTx`) RPC.
///
/// Callers can depend on:
/// - `GetClientError` means the connection was never established.
/// - `GetMempoolTxError` means the server returned a gRPC status
///   before or instead of streaming mempool transactions.
#[derive(Debug, thiserror::Error)]
pub enum GetMempoolTxError {
    #[error(transparent)]
    GetClientError(#[from] GetClientError),

    #[error("gRPC error: {0}")]
    GetMempoolTxError(#[from] tonic::Status),
}

/// Error from the `get_mempool_stream` (`GetMempoolStream`) RPC.
///
/// Callers can depend on:
/// - `GetClientError` means the connection was never established.
/// - `GetMempoolStreamError` means the server returned a gRPC status
///   before or instead of opening the stream.
#[derive(Debug, thiserror::Error)]
pub enum GetMempoolStreamError {
    #[error(transparent)]
    GetClientError(#[from] GetClientError),

    #[error("gRPC error: {0}")]
    GetMempoolStreamError(#[from] tonic::Status),
}

/// Error from the `get_latest_tree_state` (`GetLatestTreeState`) RPC.
///
/// Callers can depend on:
/// - `GetClientError` means the connection was never established.
/// - `GetLatestTreeStateError` means the server returned a gRPC status.
#[derive(Debug, thiserror::Error)]
pub enum GetLatestTreeStateError {
    #[error(transparent)]
    GetClientError(#[from] GetClientError),

    #[error("gRPC error: {0}")]
    GetLatestTreeStateError(#[from] tonic::Status),
}

/// Error from the `get_subtree_roots` (`GetSubtreeRoots`) RPC.
///
/// Callers can depend on:
/// - `GetClientError` means the connection was never established.
/// - `GetSubtreeRootsError` means the server returned a gRPC status
///   before or instead of streaming subtree roots.
#[derive(Debug, thiserror::Error)]
pub enum GetSubtreeRootsError {
    #[error(transparent)]
    GetClientError(#[from] GetClientError),

    #[error("gRPC error: {0}")]
    GetSubtreeRootsError(#[from] tonic::Status),
}

/// Error from the `ping` (`Ping`) RPC.
///
/// Callers can depend on:
/// - `GetClientError` means the connection was never established.
/// - `PingError` means the server returned a gRPC status (e.g. the
///   server was not started with `--ping-very-insecure`).
#[cfg(feature = "ping-very-insecure")]
#[derive(Debug, thiserror::Error)]
pub enum PingError {
    #[error(transparent)]
    GetClientError(#[from] GetClientError),

    #[error("gRPC error: {0}")]
    PingError(#[from] tonic::Status),
}

/// Helper to construct a `tonic::Status` for testing.
#[cfg(test)]
fn test_status() -> tonic::Status {
    tonic::Status::internal("test error")
}

/// Helper to construct a `GetClientError` for testing.
#[cfg(test)]
fn test_client_error() -> GetClientError {
    GetClientError::InvalidScheme
}

/// Macro that generates a test module for a standard 2-variant error enum
/// (GetClientError + tonic::Status).
#[cfg(test)]
macro_rules! two_variant_error_tests {
    ($mod_name:ident, $error_type:ident, $status_variant:ident) => {
        mod $mod_name {
            use super::*;

            #[test]
            fn from_get_client_error() {
                let inner = test_client_error();
                let e = $error_type::from(inner);
                assert!(matches!(e, $error_type::GetClientError(_)));
                assert!(e.to_string().contains("invalid scheme"));
            }

            #[test]
            fn from_status() {
                let status = test_status();
                let e = $error_type::from(status);
                assert!(matches!(e, $error_type::$status_variant(_)));
                assert!(e.to_string().contains("test error"));
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    two_variant_error_tests!(get_info, GetInfoError, GetLightdInfoError);
    two_variant_error_tests!(get_latest_block, GetLatestBlockError, GetLatestBlockError);
    two_variant_error_tests!(get_tree_state, GetTreeStateError, GetTreeStateError);
    two_variant_error_tests!(get_block, GetBlockError, GetBlockError);
    two_variant_error_tests!(
        get_block_nullifiers,
        GetBlockNullifiersError,
        GetBlockNullifiersError
    );
    two_variant_error_tests!(get_block_range, GetBlockRangeError, GetBlockRangeError);
    two_variant_error_tests!(
        get_block_range_nullifiers,
        GetBlockRangeNullifiersError,
        GetBlockRangeNullifiersError
    );
    two_variant_error_tests!(get_transaction, GetTransactionError, GetTransactionError);
    two_variant_error_tests!(get_mempool_tx, GetMempoolTxError, GetMempoolTxError);
    two_variant_error_tests!(
        get_mempool_stream,
        GetMempoolStreamError,
        GetMempoolStreamError
    );
    two_variant_error_tests!(
        get_latest_tree_state,
        GetLatestTreeStateError,
        GetLatestTreeStateError
    );
    two_variant_error_tests!(
        get_subtree_roots,
        GetSubtreeRootsError,
        GetSubtreeRootsError
    );

    mod send_transaction {
        use super::*;

        #[test]
        fn from_get_client_error() {
            let inner = test_client_error();
            let e = SendTransactionError::from(inner);
            assert!(matches!(e, SendTransactionError::GetClientError(_)));
        }

        #[test]
        fn from_status() {
            let status = test_status();
            let e = SendTransactionError::from(status);
            assert!(matches!(e, SendTransactionError::SendTransactionError(_)));
        }

        #[test]
        fn send_rejected() {
            let e = SendTransactionError::SendRejected("bad tx".into());
            assert!(matches!(e, SendTransactionError::SendRejected(_)));
            assert_eq!(e.to_string(), "send rejected: bad tx");
        }
    }

    #[cfg(feature = "ping-very-insecure")]
    two_variant_error_tests!(ping, PingError, PingError);
}

// ── TransparentIndexer errors ───────────────────────────────────────

#[cfg(feature = "globally-public-transparent")]
pub mod transparent {
    use super::GetClientError;

    /// Error from the deprecated `get_taddress_txids` (`GetTaddressTxids`) RPC.
    ///
    /// Callers can depend on:
    /// - `GetClientError` means the connection was never established.
    /// - `GetTaddressTxidsError` means the server returned a gRPC status.
    #[derive(Debug, thiserror::Error)]
    pub enum GetTaddressTxidsError {
        #[error(transparent)]
        GetClientError(#[from] GetClientError),

        #[error("gRPC error: {0}")]
        GetTaddressTxidsError(#[from] tonic::Status),
    }

    /// Error from the `get_taddress_transactions` (`GetTaddressTransactions`) RPC.
    ///
    /// Callers can depend on:
    /// - `GetClientError` means the connection was never established.
    /// - `GetTaddressTransactionsError` means the server returned a gRPC status.
    #[derive(Debug, thiserror::Error)]
    pub enum GetTaddressTransactionsError {
        #[error(transparent)]
        GetClientError(#[from] GetClientError),

        #[error("gRPC error: {0}")]
        GetTaddressTransactionsError(#[from] tonic::Status),
    }

    /// Error from the `get_taddress_balance` (`GetTaddressBalance`) RPC.
    ///
    /// Callers can depend on:
    /// - `GetClientError` means the connection was never established.
    /// - `GetTaddressBalanceError` means the server returned a gRPC status.
    #[derive(Debug, thiserror::Error)]
    pub enum GetTaddressBalanceError {
        #[error(transparent)]
        GetClientError(#[from] GetClientError),

        #[error("gRPC error: {0}")]
        GetTaddressBalanceError(#[from] tonic::Status),
    }

    /// Error from the `get_taddress_balance_stream` (`GetTaddressBalanceStream`) RPC.
    ///
    /// Callers can depend on:
    /// - `GetClientError` means the connection was never established.
    /// - `GetTaddressBalanceStreamError` means the server returned a gRPC status.
    #[derive(Debug, thiserror::Error)]
    pub enum GetTaddressBalanceStreamError {
        #[error(transparent)]
        GetClientError(#[from] GetClientError),

        #[error("gRPC error: {0}")]
        GetTaddressBalanceStreamError(#[from] tonic::Status),
    }

    /// Error from the `get_address_utxos` (`GetAddressUtxos`) RPC.
    ///
    /// Callers can depend on:
    /// - `GetClientError` means the connection was never established.
    /// - `GetAddressUtxosError` means the server returned a gRPC status.
    #[derive(Debug, thiserror::Error)]
    pub enum GetAddressUtxosError {
        #[error(transparent)]
        GetClientError(#[from] GetClientError),

        #[error("gRPC error: {0}")]
        GetAddressUtxosError(#[from] tonic::Status),
    }

    /// Error from the `get_address_utxos_stream` (`GetAddressUtxosStream`) RPC.
    ///
    /// Callers can depend on:
    /// - `GetClientError` means the connection was never established.
    /// - `GetAddressUtxosStreamError` means the server returned a gRPC status
    ///   before or instead of streaming UTXOs.
    #[derive(Debug, thiserror::Error)]
    pub enum GetAddressUtxosStreamError {
        #[error(transparent)]
        GetClientError(#[from] GetClientError),

        #[error("gRPC error: {0}")]
        GetAddressUtxosStreamError(#[from] tonic::Status),
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn test_status() -> tonic::Status {
            tonic::Status::internal("test error")
        }

        fn test_client_error() -> GetClientError {
            GetClientError::InvalidScheme
        }

        macro_rules! two_variant_error_tests {
            ($mod_name:ident, $error_type:ident, $status_variant:ident) => {
                mod $mod_name {
                    use super::*;

                    #[test]
                    fn from_get_client_error() {
                        let inner = test_client_error();
                        let e = $error_type::from(inner);
                        assert!(matches!(e, $error_type::GetClientError(_)));
                        assert!(e.to_string().contains("invalid scheme"));
                    }

                    #[test]
                    fn from_status() {
                        let status = test_status();
                        let e = $error_type::from(status);
                        assert!(matches!(e, $error_type::$status_variant(_)));
                        assert!(e.to_string().contains("test error"));
                    }
                }
            };
        }

        two_variant_error_tests!(
            get_taddress_txids,
            GetTaddressTxidsError,
            GetTaddressTxidsError
        );
        two_variant_error_tests!(
            get_taddress_transactions,
            GetTaddressTransactionsError,
            GetTaddressTransactionsError
        );
        two_variant_error_tests!(
            get_taddress_balance,
            GetTaddressBalanceError,
            GetTaddressBalanceError
        );
        two_variant_error_tests!(
            get_taddress_balance_stream,
            GetTaddressBalanceStreamError,
            GetTaddressBalanceStreamError
        );
        two_variant_error_tests!(
            get_address_utxos,
            GetAddressUtxosError,
            GetAddressUtxosError
        );
        two_variant_error_tests!(
            get_address_utxos_stream,
            GetAddressUtxosStreamError,
            GetAddressUtxosStreamError
        );
    }
}
