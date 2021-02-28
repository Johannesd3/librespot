//! This crate provides the [`Discovery`] struct.
//!
//! It makes this device visible to Spotify clients in the local network
//! where it is displayed in the list of "available devices". Once it is
//! selected from the list, we receive [`Credentials`] to be used in
//! [`librespot`](`librespot_core`).

#![warn(clippy::all, missing_docs, rust_2018_idioms)]

mod server;

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;
use librespot_core as core;

use self::server::DiscoveryServer;

/// Credentials to be used in [`librespot`](`librespot_core`).
pub use crate::core::authentication::Credentials;

/// Determining the icon in the list of available devices.
pub use crate::core::config::DeviceType;

/// Makes this device visible to Spotify clients in the local network.
///
/// `Discovery` implements the [`Stream`] trait. Every time this device
/// is selected in the list of available devices, it yields [`Credentials`].
pub struct Discovery {
    server: DiscoveryServer,
    _svc: libmdns::Service,
}

/// A builder for [`Discovery`].
pub struct Builder {
    server_config: server::Config,
    port: u16,
}

impl Builder {
    /// Starts a new builder using the provided device id.
    pub fn new(device_id: String) -> Self {
        Self {
            server_config: server::Config {
                name: "Librespot".into(),
                device_type: DeviceType::default(),
                device_id,
            },
            port: 0,
        }
    }

    /// Sets the name to be displayed. Default is `"Librespot"`.
    pub fn name(mut self, name: String) -> Self {
        self.server_config.name = name.into();
        self
    }

    /// Sets the device type which is visible as icon in other Spotify clients. Default is `Speaker`.
    pub fn device_type(mut self, device_type: DeviceType) -> Self {
        self.server_config.device_type = device_type;
        self
    }

    /// Sets the port on which it should listen to incoming connections.
    /// The default value `0` means any port.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets up the [`Discovery`] instance.
    ///
    /// # Errors
    /// If setting up the mdns service or creating the server fails, this function returns an error.
    pub fn launch(self) -> io::Result<Discovery> {
        Discovery::new(self)
    }
}

impl Discovery {
    /// Starts a [`Builder`] with the provided device id.
    pub fn builder(device_id: String) -> Builder {
        Builder::new(device_id)
    }

    fn new(builder: Builder) -> io::Result<Self> {
        let name = builder.server_config.name.clone();
        let mut port = builder.port;
        let server = DiscoveryServer::new(builder.server_config, &mut port)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let responder = libmdns::Responder::spawn(&tokio::runtime::Handle::current())?;

        let svc = responder.register(
            "_spotify-connect._tcp".to_owned(),
            name.into_owned(),
            port,
            &["VERSION=1.0", "CPath=/"],
        );

        Ok(Self { server, _svc: svc })
    }
}

impl Stream for Discovery {
    type Item = Credentials;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.server).poll_next(cx)
    }
}
