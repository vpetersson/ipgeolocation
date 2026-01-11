//! HTTP/3 server support using QUIC
//!
//! This module provides HTTP/3 server functionality alongside the standard HTTP/1.1+2 server.
//! HTTP/3 requires TLS certificates and runs over QUIC protocol.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use bytes::Bytes;
use h3::quic::BidiStream;
use h3::server::RequestStream;
use h3_quinn::quinn;
use http::{Request, Response, StatusCode};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio::fs;
use tracing::{error, info};

use crate::handlers::ApiDoc;
use utoipa::OpenApi;

/// Configuration for HTTP/3 server
#[derive(Clone)]
pub struct Http3Config {
    /// Address to bind to (typically same port as HTTP but UDP)
    pub bind_address: SocketAddr,
    /// Path to TLS certificate file (PEM format)
    pub cert_path: String,
    /// Path to TLS private key file (PEM format)
    pub key_path: String,
}

/// Load TLS certificates from PEM files
pub async fn load_certs(
    cert_path: &str,
    key_path: &str,
) -> Result<
    (Vec<CertificateDer<'static>>, PrivateKeyDer<'static>),
    Box<dyn std::error::Error + Send + Sync>,
> {
    let cert_pem = fs::read(cert_path).await?;
    let key_pem = fs::read(key_path).await?;

    let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut cert_pem.as_slice())
        .filter_map(|r| r.ok())
        .collect();

    let key = rustls_pemfile::private_key(&mut key_pem.as_slice())?
        .ok_or("No private key found in PEM file")?;

    Ok((certs, key))
}

/// Create QUIC server configuration with TLS
pub fn create_quic_config(
    certs: Vec<CertificateDer<'static>>,
    key: PrivateKeyDer<'static>,
) -> Result<quinn::ServerConfig, Box<dyn std::error::Error + Send + Sync>> {
    let mut tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;

    tls_config.alpn_protocols = vec![b"h3".to_vec()];

    let quic_config = quinn::ServerConfig::with_crypto(Arc::new(
        quinn::crypto::rustls::QuicServerConfig::try_from(tls_config)?,
    ));

    Ok(quic_config)
}

/// Handle an HTTP/3 request
async fn handle_request<S>(
    req: Request<()>,
    stream: &mut RequestStream<S, Bytes>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    S: BidiStream<Bytes>,
{
    let path = req.uri().path();
    let method = req.method();

    info!(
        method = %method,
        path = %path,
        "HTTP/3 request"
    );

    let (status, content_type, body) = match path {
        "/health" => (
            StatusCode::OK,
            "text/plain; charset=utf-8",
            "OK".to_string(),
        ),

        "/openapi.yaml" | "/.well-known/openapi.yaml" => {
            let spec = ApiDoc::openapi().to_yaml().unwrap();
            (StatusCode::OK, "application/yaml; charset=utf-8", spec)
        }

        "/llms.txt" => (
            StatusCode::OK,
            "text/plain; charset=utf-8",
            include_str!("../llms.txt").to_string(),
        ),

        "/sitemap.xml" => {
            // Note: This is a simplified sitemap for HTTP/3. The full sitemap
            // is served by the Axum handler which has access to base_url state.
            let sitemap = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url><loc>/</loc><priority>1.0</priority></url>
  <url><loc>/openapi.yaml</loc><priority>0.8</priority></url>
  <url><loc>/llms.txt</loc><priority>0.8</priority></url>
  <url><loc>/ipgeo</loc><priority>0.9</priority></url>
  <url><loc>/timezone</loc><priority>0.9</priority></url>
</urlset>"#;
            (
                StatusCode::OK,
                "application/xml; charset=utf-8",
                sitemap.to_string(),
            )
        }

        _ => {
            // For other endpoints, return a redirect hint to use HTTP/1.1 or HTTP/2
            // Full API functionality requires the Axum handlers with state
            let body = serde_json::json!({
                "error": "This endpoint requires HTTP/1.1 or HTTP/2",
                "hint": "HTTP/3 currently supports /health, /openapi.yaml, /.well-known/openapi.yaml, /llms.txt, and /sitemap.xml",
                "code": "HTTP3_LIMITED"
            });
            (
                StatusCode::NOT_IMPLEMENTED,
                "application/json; charset=utf-8",
                body.to_string(),
            )
        }
    };

    let response = Response::builder()
        .status(status)
        .header("content-type", content_type)
        .header("alt-svc", "h3=\":443\"; ma=86400")
        .body(())?;

    stream.send_response(response).await?;
    stream.send_data(Bytes::from(body)).await?;
    stream.finish().await?;

    Ok(())
}

/// Run the HTTP/3 server
///
/// This function starts an HTTP/3 server that runs alongside the main Axum server.
/// It requires TLS certificates and listens on UDP.
///
/// # Arguments
/// * `config` - HTTP/3 server configuration
///
/// # Example
/// ```ignore
/// let config = Http3Config {
///     bind_address: "0.0.0.0:443".parse().unwrap(),
///     cert_path: "cert.pem".to_string(),
///     key_path: "key.pem".to_string(),
/// };
/// run_http3_server(config).await?;
/// ```
pub async fn run_http3_server(
    config: Http3Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Check if certificate files exist
    if !Path::new(&config.cert_path).exists() {
        error!(
            "TLS certificate not found: {}. HTTP/3 server disabled.",
            config.cert_path
        );
        return Ok(());
    }
    if !Path::new(&config.key_path).exists() {
        error!(
            "TLS private key not found: {}. HTTP/3 server disabled.",
            config.key_path
        );
        return Ok(());
    }

    // Load TLS certificates
    let (certs, key) = load_certs(&config.cert_path, &config.key_path).await?;

    // Create QUIC configuration
    let quic_config = create_quic_config(certs, key)?;

    // Create QUIC endpoint
    let endpoint = quinn::Endpoint::server(quic_config, config.bind_address)?;

    info!(
        "HTTP/3 server listening on {} (UDP/QUIC)",
        config.bind_address
    );

    // Accept connections
    while let Some(incoming) = endpoint.accept().await {
        tokio::spawn(async move {
            match incoming.await {
                Ok(connection) => {
                    if let Err(e) = handle_connection(connection).await {
                        error!("HTTP/3 connection error: {}", e);
                    }
                }
                Err(e) => {
                    error!("HTTP/3 accept error: {}", e);
                }
            }
        });
    }

    Ok(())
}

/// Handle an HTTP/3 connection
async fn handle_connection(
    connection: quinn::Connection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut h3_conn = h3::server::Connection::new(h3_quinn::Connection::new(connection)).await?;

    loop {
        match h3_conn.accept().await {
            Ok(Some(resolver)) => {
                tokio::spawn(async move {
                    match resolver.resolve_request().await {
                        Ok((req, mut stream)) => {
                            if let Err(e) = handle_request(req, &mut stream).await {
                                error!("HTTP/3 request error: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("HTTP/3 request resolution error: {}", e);
                        }
                    }
                });
            }
            Ok(None) => {
                // Connection closed gracefully
                break;
            }
            Err(e) => {
                error!("HTTP/3 stream error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http3_config_creation() {
        let config = Http3Config {
            bind_address: "0.0.0.0:443".parse().unwrap(),
            cert_path: "cert.pem".to_string(),
            key_path: "key.pem".to_string(),
        };

        assert_eq!(config.bind_address.port(), 443);
    }

    #[test]
    fn test_accepts_protobuf_helper() {
        use crate::proto::accepts_protobuf;

        assert!(accepts_protobuf(Some("application/x-protobuf")));
        assert!(!accepts_protobuf(Some("application/json")));
    }
}
