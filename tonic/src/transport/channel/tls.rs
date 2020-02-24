use crate::transport::{
    service::TlsConnector,
    tls::{Certificate, Identity},
    Error,
};
use http::Uri;
use std::fmt;

/// Configures TLS settings for endpoints.
#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
#[derive(Clone)]
pub struct ClientTlsConfig {
    domain: Option<String>,
    cert: Option<Certificate>,
    identity: Option<Identity>,
    rustls_raw: Option<tokio_rustls::rustls::ClientConfig>,
    #[cfg(feature = "tls-dangerous")]
    certs_validation: bool,
}

#[cfg(feature = "tls")]
impl fmt::Debug for ClientTlsConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientTlsConfig")
            .field("domain", &self.domain)
            .field("cert", &self.cert)
            .field("identity", &self.identity)
            .finish()
    }
}

#[cfg(feature = "tls")]
impl ClientTlsConfig {
    /// Creates a new `ClientTlsConfig` using Rustls.
    pub fn new() -> Self {
        ClientTlsConfig {
            domain: None,
            cert: None,
            identity: None,
            rustls_raw: None,
            #[cfg(feature = "tls-dangerous")]
            certs_validation: true,
        }
    }

    /// Sets the domain name against which to verify the server's TLS certificate.
    ///
    /// This has no effect if `rustls_client_config` is used to configure Rustls.
    pub fn domain_name(self, domain_name: impl Into<String>) -> Self {
        ClientTlsConfig {
            domain: Some(domain_name.into()),
            ..self
        }
    }

    /// Sets the CA Certificate against which to verify the server's TLS certificate.
    ///
    /// This has no effect if `rustls_client_config` is used to configure Rustls.
    pub fn ca_certificate(self, ca_certificate: Certificate) -> Self {
        ClientTlsConfig {
            cert: Some(ca_certificate),
            ..self
        }
    }

    /// Sets the client identity to present to the server.
    ///
    /// This has no effect if `rustls_client_config` is used to configure Rustls.
    pub fn identity(self, identity: Identity) -> Self {
        ClientTlsConfig {
            identity: Some(identity),
            ..self
        }
    }

    /// Use options specified by the given `ClientConfig` to configure TLS.
    ///
    /// This overrides all other TLS options set via other means.
    pub fn rustls_client_config(self, config: tokio_rustls::rustls::ClientConfig) -> Self {
        ClientTlsConfig {
            rustls_raw: Some(config),
            ..self
        }
    }

    /// Disables certificate validation.
    ///
    /// # Warning
    ///
    /// You should think very carefully before using this method. If
    /// invalid certificates are trusted, *any* certificate for *any* site
    /// will be trusted for use. This includes expired certificates. This
    /// introduces significant vulnerabilities, and should only be used
    /// as a last resort.
    ///
    /// # Optional
    ///
    /// This requires the optional `tls` and `tls-dangerous` features to be enabled.
    #[cfg(feature = "tls-dangerous")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls-dangerous")))]
    pub fn danger_accept_invalid_certs(self) -> Self {
        ClientTlsConfig {
            certs_validation: true,
            ..self
        }
    }

    pub(crate) fn tls_connector(&self, uri: Uri) -> Result<TlsConnector, crate::Error> {
        let domain = match &self.domain {
            None => uri.host().ok_or(Error::new_invalid_uri())?.to_string(),
            Some(domain) => domain.clone(),
        };
        match &self.rustls_raw {
            None => {
                let certs_validation = {
                    #[cfg(feature = "tls-dangerous")]
                        {
                            self.certs_validation
                        }
                    #[cfg(not(feature = "tls-dangerous"))]
                        {
                            true
                        }
                };

                TlsConnector::new_with_rustls_cert(
                    self.cert.clone(),
                    self.identity.clone(),
                    certs_validation,
                    domain,
                )
            }
            Some(c) => TlsConnector::new_with_rustls_raw(c.clone(), domain),
        }
    }
}
