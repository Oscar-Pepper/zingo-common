//! `zingo-netutils`
//!
//! This crate provides the [`Indexer`] trait for communicating with a Zcash chain indexer,
//! and [`GrpcIndexer`], a concrete implementation that connects to a zainod server via gRPC.

use std::future::Future;
use std::time::Duration;

use tonic::Request;
use tonic::transport::{Channel, ClientTlsConfig, Endpoint};

pub use lightwallet_protocol;

use lightwallet_protocol::{
    BlockId, BlockRange, ChainSpec, CompactBlock, CompactTx, CompactTxStreamerClient, Empty,
    GetMempoolTxRequest, GetSubtreeRootsArg, LightdInfo, RawTransaction, SubtreeRoot, TreeState,
    TxFilter,
};

#[cfg(feature = "ping-very-insecure")]
use lightwallet_protocol::{Duration as ProtoDuration, PingResponse};

#[cfg(feature = "globally-public-transparent")]
mod globally_public;
#[cfg(feature = "globally-public-transparent")]
pub use globally_public::TransparentIndexer;

#[derive(Debug, thiserror::Error)]
pub enum GetClientError {
    #[error("bad uri: invalid scheme")]
    InvalidScheme,

    #[error("bad uri: invalid authority")]
    InvalidAuthority,

    #[error(transparent)]
    Transport(#[from] tonic::transport::Error),
}

fn client_tls_config() -> ClientTlsConfig {
    // Allow self-signed certs in tests
    #[cfg(test)]
    {
        ClientTlsConfig::new()
            .ca_certificate(tonic::transport::Certificate::from_pem(
                std::fs::read("test-data/localhost.pem").expect("test file"),
            ))
            .with_webpki_roots()
    }
    #[cfg(not(test))]
    ClientTlsConfig::new().with_webpki_roots()
}

const DEFAULT_GRPC_TIMEOUT: Duration = Duration::from_secs(10);

/// Error type for [`GrpcIndexer::get_info`].
#[derive(Debug, thiserror::Error)]
pub enum GetInfoError {
    #[error(transparent)]
    GetClientError(#[from] GetClientError),

    #[error("gRPC error: {0}")]
    GetLightdInfoError(#[from] tonic::Status),
}

/// Error type for [`GrpcIndexer::get_latest_block`].
#[derive(Debug, thiserror::Error)]
pub enum GetLatestBlockError {
    #[error(transparent)]
    GetClientError(#[from] GetClientError),

    #[error("gRPC error: {0}")]
    GetLatestBlockError(#[from] tonic::Status),
}

/// Error type for [`GrpcIndexer::send_transaction`].
#[derive(Debug, thiserror::Error)]
pub enum SendTransactionError {
    #[error(transparent)]
    GetClientError(#[from] GetClientError),

    #[error("gRPC error: {0}")]
    SendTransactionError(#[from] tonic::Status),

    #[error("send rejected: {0}")]
    SendRejected(String),
}

/// Error type for [`GrpcIndexer::get_tree_state`].
#[derive(Debug, thiserror::Error)]
pub enum GetTreeStateError {
    #[error(transparent)]
    GetClientError(#[from] GetClientError),

    #[error("gRPC error: {0}")]
    GetTreeStateError(#[from] tonic::Status),
}

/// Common error type for gRPC calls that only fail on connection or status.
///
/// Used by [`GrpcIndexer`] for trait methods that have no additional failure modes
/// beyond establishing a connection and receiving a gRPC response.
#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    #[error(transparent)]
    GetClientError(#[from] GetClientError),

    #[error("gRPC error: {0}")]
    Status(#[from] tonic::Status),
}

/// Trait for communicating with a Zcash chain indexer.
///
/// Implementors provide access to a lightwalletd-compatible server.
/// Callers can depend on the following guarantees:
///
/// - Each method opens a fresh connection (or reuses a pooled one) — no
///   persistent session state is assumed between calls.
/// - Errors are partitioned per method so callers can handle connection
///   failures separately from server-side errors.
/// - All methods are safe to call concurrently from multiple tasks.
pub trait Indexer {
    type GetInfoError: std::error::Error;
    type GetLatestBlockError: std::error::Error;
    type SendTransactionError: std::error::Error;
    type GetTreeStateError: std::error::Error;
    type GetBlockError: std::error::Error;
    type GetBlockNullifiersError: std::error::Error;
    type GetBlockRangeError: std::error::Error;
    type GetBlockRangeNullifiersError: std::error::Error;
    type GetTransactionError: std::error::Error;
    type GetMempoolTxError: std::error::Error;
    type GetMempoolStreamError: std::error::Error;
    type GetLatestTreeStateError: std::error::Error;
    type GetSubtreeRootsError: std::error::Error;

    #[cfg(feature = "ping-very-insecure")]
    type PingError: std::error::Error;

    /// Return server metadata (chain name, block height, version, etc.).
    ///
    /// The returned [`LightdInfo`] includes the chain name, current block height,
    /// server version, and consensus branch ID. Callers should not cache this
    /// value across sync boundaries as the block height is a point-in-time snapshot.
    fn get_info(&self) -> impl Future<Output = Result<LightdInfo, Self::GetInfoError>>;

    /// Return the height and hash of the chain tip.
    ///
    /// The returned [`BlockId`] identifies the most recent block the server
    /// is aware of. The hash may be omitted by some implementations.
    fn get_latest_block(&self) -> impl Future<Output = Result<BlockId, Self::GetLatestBlockError>>;

    /// Submit a raw transaction to the network.
    ///
    /// On success, returns the transaction ID as a hex string.
    /// On rejection by the network, returns a [`Self::SendTransactionError`]
    /// containing the rejection reason. Callers should be prepared for
    /// transient failures and may retry.
    fn send_transaction(
        &self,
        tx_bytes: Box<[u8]>,
    ) -> impl Future<Output = Result<String, Self::SendTransactionError>>;

    /// Fetch the note commitment tree state at the given block height.
    ///
    /// Returns Sapling and Orchard commitment tree frontiers as of the
    /// end of the specified block. The caller must supply a valid mined
    /// block height; requesting an unmined height is an error.
    fn get_tree_state(
        &self,
        height: u64,
    ) -> impl Future<Output = Result<TreeState, Self::GetTreeStateError>>;

    /// Return the compact block at the given height.
    ///
    /// The returned [`CompactBlock`] contains compact transaction data
    /// sufficient for trial decryption and nullifier detection.
    fn get_block(
        &self,
        block_id: BlockId,
    ) -> impl Future<Output = Result<CompactBlock, Self::GetBlockError>>;

    /// Return the compact block at the given height, containing only nullifiers.
    ///
    /// The returned [`CompactBlock`] omits output data, retaining only
    /// spend nullifiers. Callers should migrate to [`get_block`](Indexer::get_block).
    #[deprecated(note = "use get_block instead")]
    fn get_block_nullifiers(
        &self,
        block_id: BlockId,
    ) -> impl Future<Output = Result<CompactBlock, Self::GetBlockNullifiersError>>;

    /// Return a stream of consecutive compact blocks for the given range.
    ///
    /// Both endpoints of the range are inclusive. The stream yields blocks
    /// in ascending height order. Callers must consume or drop the stream
    /// before the connection is reused.
    fn get_block_range(
        &self,
        range: BlockRange,
    ) -> impl Future<Output = Result<tonic::Streaming<CompactBlock>, Self::GetBlockRangeError>>;

    /// Return a stream of consecutive compact blocks (nullifiers only) for the given range.
    ///
    /// Same streaming guarantees as [`get_block_range`](Indexer::get_block_range)
    /// but each block contains only nullifiers.
    /// Callers should migrate to [`get_block_range`](Indexer::get_block_range).
    #[deprecated(note = "use get_block_range instead")]
    fn get_block_range_nullifiers(
        &self,
        range: BlockRange,
    ) -> impl Future<Output = Result<tonic::Streaming<CompactBlock>, Self::GetBlockRangeNullifiersError>>;

    /// Return the full serialized transaction matching the given filter.
    ///
    /// The filter identifies a transaction by its txid hash. The returned
    /// [`RawTransaction`] contains the complete serialized bytes and the
    /// block height at which it was mined (0 if in the mempool).
    fn get_transaction(
        &self,
        filter: TxFilter,
    ) -> impl Future<Output = Result<RawTransaction, Self::GetTransactionError>>;

    /// Return a stream of compact transactions currently in the mempool.
    ///
    /// The request may include txid suffixes to exclude from the results,
    /// allowing the caller to avoid re-fetching known transactions.
    /// Results may be seconds out of date.
    fn get_mempool_tx(
        &self,
        request: GetMempoolTxRequest,
    ) -> impl Future<Output = Result<tonic::Streaming<CompactTx>, Self::GetMempoolTxError>>;

    /// Return a stream of raw mempool transactions.
    ///
    /// The stream remains open while there are mempool transactions and
    /// closes when a new block is mined.
    fn get_mempool_stream(
        &self,
    ) -> impl Future<Output = Result<tonic::Streaming<RawTransaction>, Self::GetMempoolStreamError>>;

    /// Return the note commitment tree state at the chain tip.
    ///
    /// Equivalent to calling [`get_tree_state`](Indexer::get_tree_state) with
    /// the current tip height, but avoids the need to query the tip first.
    fn get_latest_tree_state(
        &self,
    ) -> impl Future<Output = Result<TreeState, Self::GetLatestTreeStateError>>;

    /// Return a stream of subtree roots for the given shielded protocol.
    ///
    /// Yields roots in ascending index order starting from `start_index`.
    /// Pass `max_entries = 0` to request all available roots.
    fn get_subtree_roots(
        &self,
        arg: GetSubtreeRootsArg,
    ) -> impl Future<Output = Result<tonic::Streaming<SubtreeRoot>, Self::GetSubtreeRootsError>>;

    /// Simulate server latency for testing.
    ///
    /// The server will delay for the requested duration before responding.
    /// Returns the number of concurrent Ping RPCs at entry and exit.
    /// Requires the server to be started with `--ping-very-insecure`.
    /// Do not enable in production.
    #[cfg(feature = "ping-very-insecure")]
    fn ping(
        &self,
        duration: ProtoDuration,
    ) -> impl Future<Output = Result<PingResponse, Self::PingError>>;
}

/// gRPC-backed [`Indexer`] that connects to a lightwalletd server.
#[derive(Clone)]
pub struct GrpcIndexer {
    uri: http::Uri,
    scheme: String,
    authority: http::uri::Authority,
    endpoint: Endpoint,
}

impl std::fmt::Debug for GrpcIndexer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GrpcIndexer")
            .field("scheme", &self.scheme)
            .field("authority", &self.authority)
            .finish_non_exhaustive()
    }
}

impl GrpcIndexer {
    pub fn new(uri: http::Uri) -> Result<Self, GetClientError> {
        let scheme = uri
            .scheme_str()
            .ok_or(GetClientError::InvalidScheme)?
            .to_string();
        if scheme != "http" && scheme != "https" {
            return Err(GetClientError::InvalidScheme);
        }
        let authority = uri
            .authority()
            .ok_or(GetClientError::InvalidAuthority)?
            .clone();

        let endpoint = Endpoint::from_shared(uri.to_string())?.tcp_nodelay(true);
        let endpoint = if scheme == "https" {
            endpoint.tls_config(client_tls_config())?
        } else {
            endpoint
        };

        Ok(Self {
            uri,
            scheme,
            authority,
            endpoint,
        })
    }

    pub fn uri(&self) -> &http::Uri {
        &self.uri
    }

    /// Connect to the pre-configured endpoint and return a gRPC client.
    pub async fn get_client(&self) -> Result<CompactTxStreamerClient<Channel>, GetClientError> {
        let channel = self.endpoint.connect().await?;
        Ok(CompactTxStreamerClient::new(channel))
    }

    async fn time_boxed_call<T>(
        &self,
        payload: T,
    ) -> Result<(CompactTxStreamerClient<Channel>, Request<T>), GetClientError> {
        let client = self.get_client().await?;
        let mut request = Request::new(payload);
        request.set_timeout(DEFAULT_GRPC_TIMEOUT);
        Ok((client, request))
    }

    async fn stream_call<T>(
        &self,
        payload: T,
    ) -> Result<(CompactTxStreamerClient<Channel>, Request<T>), GetClientError> {
        let client = self.get_client().await?;
        Ok((client, Request::new(payload)))
    }
}

#[cfg(feature = "back_compatible")]
impl GrpcIndexer {
    /// Return a gRPC client using `zcash_client_backend`'s generated types,
    /// for compatibility with code that expects that crate's
    /// `CompactTxStreamerClient` (e.g. pepper-sync).
    pub async fn get_zcb_client(
        &self,
    ) -> Result<
        zcash_client_backend::proto::service::compact_tx_streamer_client::CompactTxStreamerClient<
            Channel,
        >,
        GetClientError,
    > {
        let channel = self.endpoint.connect().await?;
        Ok(
            zcash_client_backend::proto::service::compact_tx_streamer_client::CompactTxStreamerClient::new(channel),
        )
    }
}

impl Indexer for GrpcIndexer {
    type GetInfoError = GetInfoError;
    type GetLatestBlockError = GetLatestBlockError;
    type SendTransactionError = SendTransactionError;
    type GetTreeStateError = GetTreeStateError;
    type GetBlockError = RpcError;
    type GetBlockNullifiersError = RpcError;
    type GetBlockRangeError = RpcError;
    type GetBlockRangeNullifiersError = RpcError;
    type GetTransactionError = RpcError;
    type GetMempoolTxError = RpcError;
    type GetMempoolStreamError = RpcError;
    type GetLatestTreeStateError = RpcError;
    type GetSubtreeRootsError = RpcError;
    #[cfg(feature = "ping-very-insecure")]
    type PingError = RpcError;

    async fn get_info(&self) -> Result<LightdInfo, GetInfoError> {
        let (mut client, request) = self.time_boxed_call(Empty {}).await?;
        Ok(client.get_lightd_info(request).await?.into_inner())
    }

    async fn get_latest_block(&self) -> Result<BlockId, GetLatestBlockError> {
        let (mut client, request) = self.time_boxed_call(ChainSpec {}).await?;
        Ok(client.get_latest_block(request).await?.into_inner())
    }

    async fn send_transaction(&self, tx_bytes: Box<[u8]>) -> Result<String, SendTransactionError> {
        let (mut client, request) = self
            .time_boxed_call(RawTransaction {
                data: tx_bytes.to_vec(),
                height: 0,
            })
            .await?;
        let sendresponse = client.send_transaction(request).await?.into_inner();
        if sendresponse.error_code == 0 {
            let mut transaction_id = sendresponse.error_message;
            if transaction_id.starts_with('\"') && transaction_id.ends_with('\"') {
                transaction_id = transaction_id[1..transaction_id.len() - 1].to_string();
            }
            Ok(transaction_id)
        } else {
            Err(SendTransactionError::SendRejected(format!(
                "{sendresponse:?}"
            )))
        }
    }

    async fn get_tree_state(&self, height: u64) -> Result<TreeState, GetTreeStateError> {
        let (mut client, request) = self
            .time_boxed_call(BlockId {
                height,
                hash: vec![],
            })
            .await?;
        Ok(client.get_tree_state(request).await?.into_inner())
    }

    async fn get_block(&self, block_id: BlockId) -> Result<CompactBlock, RpcError> {
        let (mut client, request) = self.time_boxed_call(block_id).await?;
        Ok(client.get_block(request).await?.into_inner())
    }

    #[allow(deprecated)]
    async fn get_block_nullifiers(&self, block_id: BlockId) -> Result<CompactBlock, RpcError> {
        let (mut client, request) = self.time_boxed_call(block_id).await?;
        Ok(client.get_block_nullifiers(request).await?.into_inner())
    }

    async fn get_block_range(
        &self,
        range: BlockRange,
    ) -> Result<tonic::Streaming<CompactBlock>, RpcError> {
        let (mut client, request) = self.stream_call(range).await?;
        Ok(client.get_block_range(request).await?.into_inner())
    }

    #[allow(deprecated)]
    async fn get_block_range_nullifiers(
        &self,
        range: BlockRange,
    ) -> Result<tonic::Streaming<CompactBlock>, RpcError> {
        let (mut client, request) = self.stream_call(range).await?;
        Ok(client
            .get_block_range_nullifiers(request)
            .await?
            .into_inner())
    }

    async fn get_transaction(&self, filter: TxFilter) -> Result<RawTransaction, RpcError> {
        let (mut client, request) = self.time_boxed_call(filter).await?;
        Ok(client.get_transaction(request).await?.into_inner())
    }

    async fn get_mempool_tx(
        &self,
        request: GetMempoolTxRequest,
    ) -> Result<tonic::Streaming<CompactTx>, RpcError> {
        let (mut client, request) = self.stream_call(request).await?;
        Ok(client.get_mempool_tx(request).await?.into_inner())
    }

    async fn get_mempool_stream(&self) -> Result<tonic::Streaming<RawTransaction>, RpcError> {
        let (mut client, request) = self.stream_call(Empty {}).await?;
        Ok(client.get_mempool_stream(request).await?.into_inner())
    }

    async fn get_latest_tree_state(&self) -> Result<TreeState, RpcError> {
        let (mut client, request) = self.time_boxed_call(Empty {}).await?;
        Ok(client.get_latest_tree_state(request).await?.into_inner())
    }

    async fn get_subtree_roots(
        &self,
        arg: GetSubtreeRootsArg,
    ) -> Result<tonic::Streaming<SubtreeRoot>, RpcError> {
        let (mut client, request) = self.stream_call(arg).await?;
        Ok(client.get_subtree_roots(request).await?.into_inner())
    }

    #[cfg(feature = "ping-very-insecure")]
    async fn ping(&self, duration: ProtoDuration) -> Result<PingResponse, RpcError> {
        let (mut client, request) = self.time_boxed_call(duration).await?;
        Ok(client.ping(request).await?.into_inner())
    }
}

#[cfg(test)]
mod indexer_implementation {

    mod get_info {
        #[tokio::test]
        async fn call_get_info() {
            assert_eq!(1, 1);
            //let grpc_index = GrpcIndexer::new();
        }
    }
}

#[cfg(test)]
mod tests {
    //! Unit and integration-style tests for `zingo-netutils`.
    //!
    //! These tests focus on:
    //! - TLS test asset sanity (`test-data/localhost.pem` + `.key`)
    //! - Rustls plumbing (adding a local cert to a root store)
    //! - Connector correctness (scheme validation, HTTP/2 expectations)
    //! - URI rewrite behavior (no panics; returns structured errors)
    //!
    //! Notes:
    //! - Some tests spin up an in-process TLS server and use aggressive timeouts to
    //!   avoid hangs under nextest.
    //! - We explicitly install a rustls crypto provider to avoid
    //!   provider-selection panics in test binaries.

    use std::time::Duration;

    use http::{Request, Response};
    use hyper::{
        body::{Bytes, Incoming},
        service::service_fn,
    };
    use hyper_util::rt::TokioIo;
    use tokio::{net::TcpListener, sync::oneshot, time::timeout};
    use tokio_rustls::{TlsAcceptor, rustls};

    use super::*;

    use tokio_rustls::rustls::RootCertStore;

    fn add_test_cert_to_roots(roots: &mut RootCertStore) {
        use tonic::transport::CertificateDer;
        eprintln!("Adding test cert to roots");

        const TEST_PEMFILE_PATH: &str = "test-data/localhost.pem";

        let Ok(fd) = std::fs::File::open(TEST_PEMFILE_PATH) else {
            eprintln!("Test TLS cert not found at {TEST_PEMFILE_PATH}, skipping");
            return;
        };

        let mut buf = std::io::BufReader::new(fd);
        let certs_bytes: Vec<tonic::transport::CertificateDer> = rustls_pemfile::certs(&mut buf)
            .filter_map(Result::ok)
            .collect();

        let certs: Vec<CertificateDer<'_>> = certs_bytes.into_iter().collect();
        roots.add_parsable_certificates(certs);
    }

    /// Ensures the committed localhost test certificate exists and is parseable as X.509.
    ///
    /// This catches:
    /// - missing file / wrong working directory assumptions
    /// - invalid PEM encoding
    /// - accidentally committing the wrong artifact (e.g., key instead of cert)
    #[test]
    fn localhost_cert_file_exists_and_is_parseable() {
        const CERT_PATH: &str = "test-data/localhost.pem";

        let pem = std::fs::read(CERT_PATH).expect("missing test-data/localhost.pem");

        let mut cursor = std::io::BufReader::new(pem.as_slice());
        let certs = rustls_pemfile::certs(&mut cursor)
            .filter_map(Result::ok)
            .collect::<Vec<_>>();

        assert!(!certs.is_empty(), "no certs found in {CERT_PATH}");

        for cert in certs {
            let der = cert.as_ref();
            let parsed = x509_parser::parse_x509_certificate(der);
            assert!(
                parsed.is_ok(),
                "failed to parse a cert from {CERT_PATH} as X.509"
            );
        }
    }

    /// Guards against committing a CA certificate as the TLS server certificate.
    ///
    /// Rustls rejects certificates with CA constraints when used as an end-entity
    /// server certificate (e.g. `CaUsedAsEndEntity`), even if the cert is in the
    /// root store. This test ensures the committed localhost cert has `CA:FALSE`.
    #[test]
    fn localhost_cert_is_end_entity_not_ca() {
        let pem =
            std::fs::read("test-data/localhost.pem").expect("missing test-data/localhost.pem");
        let mut cursor = std::io::BufReader::new(pem.as_slice());

        let certs = rustls_pemfile::certs(&mut cursor)
            .filter_map(Result::ok)
            .collect::<Vec<_>>();

        assert!(!certs.is_empty(), "no certs found in localhost.pem");

        let der = certs[0].as_ref();
        let parsed = x509_parser::parse_x509_certificate(der).expect("failed to parse X.509");
        let x509 = parsed.1;

        let constraints = x509
            .basic_constraints()
            .expect("missing basic constraints extension");

        assert!(
            !constraints.unwrap().value.ca,
            "localhost.pem must be CA:FALSE"
        );
    }

    /// Loads a rustls `ServerConfig` for a local TLS server using the committed
    /// test certificate and private key.
    ///
    /// The cert/key pair is *test-only* and is stored under `test-data/`.
    /// This is used to verify that the client-side root-store injection
    /// (`add_test_cert_to_roots`) actually enables successful TLS handshakes.
    fn load_test_server_config() -> std::sync::Arc<rustls::ServerConfig> {
        let cert_pem =
            std::fs::read("test-data/localhost.pem").expect("missing test-data/localhost.pem");
        let key_pem =
            std::fs::read("test-data/localhost.key").expect("missing test-data/localhost.key");

        let mut cert_cursor = std::io::BufReader::new(cert_pem.as_slice());
        let mut key_cursor = std::io::BufReader::new(key_pem.as_slice());

        let certs = rustls_pemfile::certs(&mut cert_cursor)
            .filter_map(Result::ok)
            .map(rustls::pki_types::CertificateDer::from)
            .collect::<Vec<_>>();

        let key = rustls_pemfile::private_key(&mut key_cursor)
            .expect("failed to read private key")
            .expect("no private key found");

        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .expect("bad cert or key");

        std::sync::Arc::new(config)
    }
    /// Smoke test: adding the committed localhost cert to a rustls root store enables
    /// a client to complete a TLS handshake and perform an HTTP request.
    ///
    /// Implementation notes:
    /// - Uses a local TLS server with the committed cert/key.
    /// - Uses strict timeouts to prevent hangs under nextest.
    /// - Explicitly drains the request body and disables keep-alive so that
    ///   `serve_connection` terminates deterministically.
    /// - Installs the rustls crypto provider to avoid provider
    ///   selection panics in test binaries.
    #[tokio::test]
    async fn add_test_cert_to_roots_enables_tls_handshake() {
        use http_body_util::Full;
        use hyper::service::service_fn;
        use hyper_util::rt::TokioIo;
        use tokio::net::TcpListener;
        use tokio_rustls::TlsAcceptor;
        use tokio_rustls::rustls;

        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind failed");
        let addr = listener.local_addr().expect("local_addr failed");

        let tls_config = load_test_server_config();
        let acceptor = TlsAcceptor::from(tls_config);

        let ready = oneshot::channel::<()>();
        let ready_tx = ready.0;
        let ready_rx = ready.1;

        let server_task = tokio::spawn(async move {
            let _ = ready_tx.send(());

            let accept_res = timeout(Duration::from_secs(3), listener.accept()).await;
            let (socket, _) = accept_res
                .expect("server accept timed out")
                .expect("accept failed");

            let tls_stream = timeout(Duration::from_secs(3), acceptor.accept(socket))
                .await
                .expect("tls accept timed out")
                .expect("tls accept failed");

            let io = TokioIo::new(tls_stream);

            let svc = service_fn(|mut req: http::Request<hyper::body::Incoming>| async move {
                use http_body_util::BodyExt;

                while let Some(frame) = req.body_mut().frame().await {
                    if frame.is_err() {
                        break;
                    }
                }

                let mut resp = http::Response::new(Full::new(Bytes::from_static(b"ok")));
                resp.headers_mut().insert(
                    http::header::CONNECTION,
                    http::HeaderValue::from_static("close"),
                );
                Ok::<_, hyper::Error>(resp)
            });

            timeout(
                Duration::from_secs(3),
                hyper::server::conn::http1::Builder::new()
                    .keep_alive(false)
                    .serve_connection(io, svc),
            )
            .await
            .expect("serve_connection timed out")
            .expect("serve_connection failed");
        });

        let _ = timeout(Duration::from_secs(1), ready_rx)
            .await
            .expect("server ready signal timed out")
            .expect("server dropped before ready");

        // Build client root store and add the test cert.
        let mut roots = rustls::RootCertStore::empty();
        add_test_cert_to_roots(&mut roots);

        let client_config = rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();

        // This MUST allow http1 since the server uses hyper http1 builder.
        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_tls_config(client_config)
            .https_only()
            .enable_http1()
            .build();

        let client =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build(https);

        let uri: http::Uri = format!("https://127.0.0.1:{}/", addr.port())
            .parse()
            .expect("bad uri");

        let req = http::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Full::<Bytes>::new(Bytes::new()))
            .expect("request build failed");

        let res = timeout(Duration::from_secs(3), client.request(req))
            .await
            .expect("client request timed out")
            .expect("TLS handshake or request failed");

        assert!(res.status().is_success());

        timeout(Duration::from_secs(3), server_task)
            .await
            .expect("server task timed out")
            .expect("server task failed");
    }

    /// Validates that the connector rejects non-HTTP(S) URIs.
    ///
    /// This test is intended to fail until production code checks for:
    /// - `http` and `https` schemes only
    /// and rejects everything else (e.g. `ftp`).
    #[test]
    fn rejects_non_http_schemes() {
        let uri: http::Uri = "ftp://example.com:1234".parse().unwrap();
        let res = GrpcIndexer::new(uri);

        assert!(
            res.is_err(),
            "expected GrpcIndexer::new() to reject non-http(s) schemes, but got Ok"
        );
    }

    /// Demonstrates the HTTPS downgrade hazard: the underlying client can successfully
    /// talk to an HTTP/1.1-only TLS server if the HTTPS branch does not enforce HTTP/2.
    ///
    /// This is intentionally written as a “should be HTTP/2” test so it fails until
    /// the HTTPS client is constructed with `http2_only(true)`.
    #[tokio::test]
    async fn https_connector_must_not_downgrade_to_http1() {
        use http_body_util::Full;

        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind failed");
        let addr = listener.local_addr().expect("local_addr failed");

        let tls_config = load_test_server_config();
        let acceptor = TlsAcceptor::from(tls_config);

        let server_task = tokio::spawn(async move {
            let accept_res = timeout(Duration::from_secs(3), listener.accept()).await;
            let (socket, _) = accept_res
                .expect("server accept timed out")
                .expect("accept failed");

            let tls_stream = acceptor.accept(socket).await.expect("tls accept failed");
            let io = TokioIo::new(tls_stream);

            let svc = service_fn(|_req: Request<Incoming>| async move {
                Ok::<_, hyper::Error>(Response::new(Full::new(Bytes::from_static(b"ok"))))
            });

            // This may error with VersionH2 if the client sends an h2 preface, or it may
            // simply never be reached if ALPN fails earlier. Either is fine for this test.
            let _ = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, svc)
                .await;
        });

        let base = format!("https://127.0.0.1:{}", addr.port());
        let uri = base.parse::<http::Uri>().expect("bad base uri");

        let endpoint = tonic::transport::Endpoint::from_shared(uri.to_string())
            .expect("endpoint")
            .tcp_nodelay(true);

        let connect_res = endpoint
            .tls_config(client_tls_config())
            .expect("tls_config failed")
            .connect()
            .await;

        // A gRPC (HTTP/2) client must not succeed against an HTTP/1.1-only TLS server.
        assert!(
            connect_res.is_err(),
            "expected connect to fail (no downgrade to HTTP/1.1), but it succeeded"
        );

        server_task.abort();
    }

    #[tokio::test]
    async fn connects_to_public_mainnet_indexer_and_gets_info() {
        let endpoint = "https://zec.rocks:443".to_string();

        let uri: http::Uri = endpoint.parse().expect("bad mainnet indexer URI");

        let response = GrpcIndexer::new(uri)
            .expect("URI to be valid.")
            .get_info()
            .await
            .expect("to get info");
        assert!(
            !response.chain_name.is_empty(),
            "chain_name should not be empty"
        );
        assert!(
            response.block_height > 0,
            "block_height should be > 0, got {}",
            response.block_height
        );

        let chain = response.chain_name.to_ascii_lowercase();
        assert!(
            chain.contains("main"),
            "expected a mainnet server, got chain_name={:?}",
            response.chain_name
        );
    }
}
