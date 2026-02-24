//! `zingo-netutils`
//!
//! This crate provides the `GrpcConnector` struct,
//! used to communicate with an indexer.

use http::Uri;
#[cfg(test)]
use tokio_rustls::rustls::RootCertStore;
use tonic::transport::{Channel, ClientTlsConfig, Endpoint};
use zcash_client_backend::proto::service::compact_tx_streamer_client::CompactTxStreamerClient;

#[derive(Debug, thiserror::Error)]
pub enum GetClientError {
    #[error("bad uri: invalid scheme")]
    InvalidScheme,

    #[error("bad uri: invalid authority")]
    InvalidAuthority,

    #[error("bad uri: invalid path and/or query")]
    InvalidPathAndQuery,

    #[error(transparent)]
    Transport(#[from] tonic::transport::Error),
}

pub mod client {
    use http_body::Body;
    use hyper_util::client::legacy::{Client, connect::Connect};

    /// A utility used in multiple places
    pub fn client_from_connector<C, B>(connector: C, http2_only: bool) -> Box<Client<C, B>>
    where
        C: Connect + Clone,
        B: Body + Send,
        B::Data: Send,
    {
        Box::new(
            Client::builder(hyper_util::rt::TokioExecutor::new())
                .http2_only(http2_only)
                .build(connector),
        )
    }
}

#[cfg(test)]
fn load_test_cert_pem() -> Option<Vec<u8>> {
    const TEST_PEMFILE_PATH: &str = "test-data/localhost.pem";
    std::fs::read(TEST_PEMFILE_PATH).ok()
}
fn client_tls_config() -> Result<ClientTlsConfig, GetClientError> {
    // Allow self-signed certs in tests
    #[cfg(test)]
    {
        if let Some(pem) = load_test_cert_pem() {
            return Ok(
                ClientTlsConfig::new().ca_certificate(tonic::transport::Certificate::from_pem(pem))
            );
        }
    }

    Ok(ClientTlsConfig::new())
}
/// The connector, containing the URI to connect to.
/// This type is mostly an interface to the `get_client` method.
/// The proto-generated `CompactTxStreamerClient` type is the main
/// interface to actually communicating with a Zcash indexer.
#[derive(Clone)]
pub struct GrpcConnector {
    uri: http::Uri,
}

impl GrpcConnector {
    /// Takes a URI, and wraps in a `GrpcConnector`
    pub fn new(uri: http::Uri) -> Self {
        Self { uri }
    }

    /// The URI to connect to.
    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    /// Connect to the URI, and return a Client. For the full list of methods
    /// the client supports, see the service.proto file (some of the types
    /// are defined in the `compact_formats.proto` file).
    pub async fn get_client(&self) -> Result<CompactTxStreamerClient<Channel>, GetClientError> {
        let scheme = self.uri.scheme_str().ok_or(GetClientError::InvalidScheme)?;
        if scheme != "http" && scheme != "https" {
            return Err(GetClientError::InvalidScheme);
        }
        let _authority = self
            .uri
            .authority()
            .ok_or(GetClientError::InvalidAuthority)?;

        let endpoint = Endpoint::from_shared(self.uri.to_string())?.tcp_nodelay(true);

        let channel = if scheme == "https" {
            let tls = client_tls_config()?;
            endpoint.tls_config(tls)?.connect().await?
        } else {
            endpoint.connect().await?
        };

        Ok(CompactTxStreamerClient::new(channel))
    }
}

#[cfg(test)]
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

    use http::{Request, Response, uri::PathAndQuery};
    use hyper::{
        body::{Bytes, Incoming},
        service::service_fn,
    };
    use hyper_util::rt::TokioIo;
    use tokio::{net::TcpListener, sync::oneshot, time::timeout};
    use tokio_rustls::{TlsAcceptor, rustls};

    use super::*;

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

        let _ = rustls::crypto::ring::default_provider().install_default();

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
    #[tokio::test]
    async fn rejects_non_http_schemes() {
        let uri: http::Uri = "ftp://example.com:1234".parse().unwrap();
        let connector = GrpcConnector::new(uri);
        let res = connector.get_client().await;

        assert!(
            res.is_err(),
            "expected get_client() to reject non-http(s) schemes, but got Ok"
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

        let _ = rustls::crypto::ring::default_provider().install_default();

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

        let tls = client_tls_config().expect("tls config");
        let connect_res = endpoint
            .tls_config(tls)
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

    /// Rewrites a request URI by injecting a base `scheme://authority` and a
    /// request-provided path.
    ///
    /// This is a test helper to validate the intended error behavior for
    /// malformed inputs. The production code currently uses `unwrap()` when
    /// rebuilding the URI. Tests use this helper to lock in a “no panics,
    /// return `InvalidPathAndQuery`” contract for the eventual refactor.
    fn rewrite_request_uri(
        scheme: &str,
        authority: &str,
        path_and_query: &str,
    ) -> Result<Uri, GetClientError> {
        Uri::builder()
            .scheme(scheme)
            .authority(authority)
            .path_and_query(path_and_query)
            .build()
            .map_err(|_| GetClientError::InvalidPathAndQuery)
    }
    /// Ensures URI rewriting returns a structured error for invalid inputs instead
    /// of panicking.
    ///
    /// This is a forward-looking regression test for refactoring production code to
    /// replace `unwrap()` with `map_err(|_| InvalidPathAndQuery)`.
    #[test]
    fn rewrite_returns_error_instead_of_panicking() {
        // Intentionally invalid path and query string.
        let bad = "not-a path";

        let result = rewrite_request_uri("https", "example.com:443", bad);

        assert!(matches!(result, Err(GetClientError::InvalidPathAndQuery)));

        let _ = PathAndQuery::from_static("/");
    }
}
