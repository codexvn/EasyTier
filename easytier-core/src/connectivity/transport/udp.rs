use std::{net::SocketAddr, sync::Arc, time::Duration};

use futures::{StreamExt, stream, stream::FuturesUnordered};
use url::Url;

use crate::{
    connectivity::protocol::ClientProtocolUpgrader,
    socket::{
        IpVersion,
        tcp::VirtualTcpSocketFactory,
        udp::{
            UdpBindOptions, UdpSession, UdpSessionLayer, UdpSessionProtocol,
            VirtualUdpSocketFactory,
        },
    },
    tunnel::Tunnel,
};

use super::{ConnectedTransport, first_success};

const MAX_QUIC_CANDIDATES_IN_FLIGHT: usize = 8;

pub struct ConnectedUdpSession {
    session: UdpSession,
    keep_alive: Box<dyn Send + Sync>,
}

impl ConnectedUdpSession {
    pub fn new<T>(session: UdpSession, keep_alive: T) -> Self
    where
        T: Send + Sync + 'static,
    {
        Self {
            session,
            keep_alive: Box::new(keep_alive),
        }
    }

    pub fn session(&self) -> &UdpSession {
        &self.session
    }

    pub fn into_parts(self) -> (UdpSession, Box<dyn Send + Sync>) {
        (self.session, self.keep_alive)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UdpSessionMode {
    EasyTierMux,
    Classified(UdpSessionProtocol),
}

pub async fn connect_udp<H>(
    host: Arc<H>,
    remote_addr: SocketAddr,
    bind_addrs: Vec<SocketAddr>,
    default_bind: UdpBindOptions,
    mode: UdpSessionMode,
) -> anyhow::Result<ConnectedUdpSession>
where
    H: VirtualUdpSocketFactory,
{
    let ip_version = if remote_addr.is_ipv4() {
        IpVersion::V4
    } else {
        IpVersion::V6
    };
    let default_bind = default_bind.with_ip_version(ip_version);
    let futures = FuturesUnordered::new();
    if bind_addrs.is_empty() {
        let local_addr = if remote_addr.is_ipv4() {
            "0.0.0.0:0".parse().expect("static IPv4 bind address")
        } else {
            "[::]:0".parse().expect("static IPv6 bind address")
        };
        let bind = default_bind
            .with_local_addr(Some(local_addr))
            .with_only_v6(true);
        futures.push(bind_and_connect(host.clone(), bind, remote_addr, mode));
    } else {
        for bind_addr in bind_addrs {
            let bind = default_bind
                .clone()
                .with_local_addr(Some(bind_addr))
                .with_only_v6(true);
            futures.push(bind_and_connect(host.clone(), bind, remote_addr, mode));
        }
    }
    first_success(futures).await
}

pub(crate) async fn connect_quic_candidates<H>(
    host: Arc<H>,
    protocol: Arc<dyn ClientProtocolUpgrader<<H as VirtualTcpSocketFactory>::Socket>>,
    remote_addr: SocketAddr,
    bind_addrs: Vec<SocketAddr>,
    bind: UdpBindOptions,
    url: Url,
    time_budget: Duration,
) -> anyhow::Result<Box<dyn Tunnel>>
where
    H: VirtualUdpSocketFactory + VirtualTcpSocketFactory,
{
    let candidates = if bind_addrs.is_empty() {
        vec![None]
    } else {
        bind_addrs.into_iter().map(Some).collect()
    };
    let candidate_count = candidates.len();
    let batches = candidate_count.div_ceil(MAX_QUIC_CANDIDATES_IN_FLIGHT);
    let candidate_timeout = (batches > 1).then(|| time_budget / batches as u32);
    let attempts = stream::iter(candidates.into_iter().enumerate().map(|(index, addr)| {
        let host = host.clone();
        let protocol = protocol.clone();
        let bind = bind.clone();
        let url = url.clone();
        async move {
            let attempt = async {
            let session = match connect_udp(
                host,
                remote_addr,
                addr.into_iter().collect(),
                bind,
                UdpSessionMode::Classified(UdpSessionProtocol::Quic),
            )
            .await
            {
                Ok(session) => session,
                Err(error) => {
                    let io_error = error.chain().find_map(|cause| cause.downcast_ref::<std::io::Error>());
                    tracing::debug!(candidate = index + 1, io_error_kind = ?io_error.map(|error| error.kind()), os_error = ?io_error.and_then(|error| error.raw_os_error()), "QUIC candidate transport failed");
                    return Err(error);
                }
            };
            match protocol
                .upgrade_client(ConnectedTransport::Udp(session), url)
                .await
            {
                Ok(tunnel) => {
                    tracing::debug!(
                        candidate = index + 1,
                        candidate_count,
                        "QUIC candidate connected"
                    );
                    Ok(tunnel)
                }
                Err(error) => {
                    let io_error = error.chain().find_map(|cause| cause.downcast_ref::<std::io::Error>());
                    tracing::debug!(candidate = index + 1, io_error_kind = ?io_error.map(|error| error.kind()), os_error = ?io_error.and_then(|error| error.raw_os_error()), "QUIC candidate handshake failed");
                    Err(error)
                }
            }
            };
            if let Some(timeout) = candidate_timeout {
                match crate::foundation::time::timeout(timeout, attempt).await {
                    Ok(result) => result,
                    Err(_) => {
                        tracing::debug!(candidate = index + 1, "QUIC candidate timed out");
                        Err(anyhow::anyhow!("QUIC candidate timed out"))
                    }
                }
            } else {
                attempt.await
            }
        }
    }))
    .buffer_unordered(MAX_QUIC_CANDIDATES_IN_FLIGHT);
    futures::pin_mut!(attempts);
    let mut last_error = None;
    while let Some(result) = attempts.next().await {
        match result {
            Ok(tunnel) => return Ok(tunnel),
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("no QUIC transport candidates")))
}

async fn bind_and_connect<H>(
    host: Arc<H>,
    bind: UdpBindOptions,
    remote_addr: SocketAddr,
    mode: UdpSessionMode,
) -> anyhow::Result<ConnectedUdpSession>
where
    H: VirtualUdpSocketFactory,
{
    let socket = host.bind_udp(bind).await?;
    let layer = Arc::new(UdpSessionLayer::new_with_stun_responder(socket, host));
    let session = match mode {
        UdpSessionMode::EasyTierMux => layer.connect(remote_addr).await?,
        UdpSessionMode::Classified(protocol) => {
            layer.open_classified_session(protocol, remote_addr)?
        }
    };
    Ok(ConnectedUdpSession::new(session, layer))
}

#[cfg(test)]
mod tests {
    use std::{
        net::IpAddr,
        sync::atomic::{AtomicUsize, Ordering},
        time::Duration,
    };

    use async_trait::async_trait;

    use super::*;
    use crate::{
        host::testkit::{TestHost, TestTcpSocket},
        proto::common::TunnelInfo,
        socket::udp::UdpSessionSocket,
        tunnel::SplitTunnel,
    };

    struct TestTunnel;

    impl Tunnel for TestTunnel {
        fn split(&self) -> SplitTunnel {
            unreachable!()
        }

        fn info(&self) -> Option<TunnelInfo> {
            None
        }
    }

    struct TestQuicUpgrader {
        reachable: Option<IpAddr>,
        attempted: AtomicUsize,
    }

    #[async_trait]
    impl ClientProtocolUpgrader<TestTcpSocket> for TestQuicUpgrader {
        fn supports_scheme(&self, scheme: &str) -> bool {
            scheme == "quic"
        }

        async fn upgrade_client(
            &self,
            connected: ConnectedTransport<TestTcpSocket>,
            _url: Url,
        ) -> anyhow::Result<Box<dyn Tunnel>> {
            let ConnectedTransport::Udp(session) = connected else {
                anyhow::bail!("expected UDP session");
            };
            self.attempted.fetch_add(1, Ordering::Relaxed);
            let local_ip = session.session().local_addr()?.ip();
            if Some(local_ip) != self.reachable {
                anyhow::bail!("unreachable interface");
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok(Box::new(TestTunnel))
        }
    }

    #[tokio::test]
    async fn quic_tries_other_interfaces_after_a_handshake_failure() {
        let host = Arc::new(TestHost::default());
        let protocol = Arc::new(TestQuicUpgrader {
            reachable: Some("192.0.2.2".parse().unwrap()),
            attempted: AtomicUsize::new(0),
        });
        let result = connect_quic_candidates(
            host,
            protocol.clone(),
            "192.0.2.100:11012".parse().unwrap(),
            vec![
                "192.0.2.1:0".parse().unwrap(),
                "192.0.2.2:0".parse().unwrap(),
            ],
            UdpBindOptions::direct_connect(),
            "quic://example.com:11012".parse().unwrap(),
            Duration::from_secs(2),
        )
        .await;
        assert!(result.is_ok());
        assert_eq!(protocol.attempted.load(Ordering::Relaxed), 2);
    }

    #[tokio::test]
    async fn quic_reports_failure_after_all_candidates_fail() {
        let protocol = Arc::new(TestQuicUpgrader {
            reachable: None,
            attempted: AtomicUsize::new(0),
        });
        let result = connect_quic_candidates(
            Arc::new(TestHost::default()),
            protocol.clone(),
            "192.0.2.100:11012".parse().unwrap(),
            vec![
                "192.0.2.1:0".parse().unwrap(),
                "192.0.2.2:0".parse().unwrap(),
            ],
            UdpBindOptions::direct_connect(),
            "quic://example.com:11012".parse().unwrap(),
            Duration::from_secs(2),
        )
        .await;
        assert!(result.is_err());
        assert_eq!(protocol.attempted.load(Ordering::Relaxed), 2);
    }

    #[tokio::test]
    async fn quic_uses_wildcard_when_no_interface_is_selected() {
        let protocol = Arc::new(TestQuicUpgrader {
            reachable: Some("0.0.0.0".parse().unwrap()),
            attempted: AtomicUsize::new(0),
        });
        let result = connect_quic_candidates(
            Arc::new(TestHost::default()),
            protocol.clone(),
            "192.0.2.100:11012".parse().unwrap(),
            Vec::new(),
            UdpBindOptions::direct_connect(),
            "quic://example.com:11012".parse().unwrap(),
            Duration::from_secs(2),
        )
        .await;
        assert!(result.is_ok());
        assert_eq!(protocol.attempted.load(Ordering::Relaxed), 1);
    }

    struct PendingQuicUpgrader {
        active: Arc<AtomicUsize>,
        started: AtomicUsize,
        reachable: Option<IpAddr>,
    }

    struct ActiveAttempt(Arc<AtomicUsize>);

    impl Drop for ActiveAttempt {
        fn drop(&mut self) {
            self.0.fetch_sub(1, Ordering::Relaxed);
        }
    }

    #[async_trait]
    impl ClientProtocolUpgrader<TestTcpSocket> for PendingQuicUpgrader {
        fn supports_scheme(&self, scheme: &str) -> bool {
            scheme == "quic"
        }

        async fn upgrade_client(
            &self,
            connected: ConnectedTransport<TestTcpSocket>,
            _url: Url,
        ) -> anyhow::Result<Box<dyn Tunnel>> {
            self.active.fetch_add(1, Ordering::Relaxed);
            let _attempt = ActiveAttempt(self.active.clone());
            self.started.fetch_add(1, Ordering::Relaxed);
            let ConnectedTransport::Udp(session) = connected else {
                anyhow::bail!("expected UDP session");
            };
            if Some(session.session().local_addr()?.ip()) == self.reachable {
                return Ok(Box::new(TestTunnel));
            }
            std::future::pending().await
        }
    }

    #[tokio::test]
    async fn quic_limits_and_cancels_in_flight_candidates() {
        let active = Arc::new(AtomicUsize::new(0));
        let protocol = Arc::new(PendingQuicUpgrader {
            active: active.clone(),
            started: AtomicUsize::new(0),
            reachable: None,
        });
        let candidates = (1..=12)
            .map(|last| SocketAddr::from(([192, 0, 2, last], 0)))
            .collect();
        let task = tokio::spawn(connect_quic_candidates(
            Arc::new(TestHost::default()),
            protocol.clone(),
            "192.0.2.100:11012".parse().unwrap(),
            candidates,
            UdpBindOptions::direct_connect(),
            "quic://example.com:11012".parse().unwrap(),
            Duration::from_secs(2),
        ));
        tokio::time::timeout(Duration::from_secs(1), async {
            while protocol.started.load(Ordering::Relaxed) < MAX_QUIC_CANDIDATES_IN_FLIGHT {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert_eq!(
            active.load(Ordering::Relaxed),
            MAX_QUIC_CANDIDATES_IN_FLIGHT
        );
        assert_eq!(
            protocol.started.load(Ordering::Relaxed),
            MAX_QUIC_CANDIDATES_IN_FLIGHT
        );
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        assert_eq!(active.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn quic_attempts_queued_candidates_when_first_batch_stalls() {
        let protocol = Arc::new(PendingQuicUpgrader {
            active: Arc::new(AtomicUsize::new(0)),
            started: AtomicUsize::new(0),
            reachable: Some("192.0.2.9".parse().unwrap()),
        });
        let candidates = (1..=9)
            .map(|last| SocketAddr::from(([192, 0, 2, last], 0)))
            .collect();
        let result = tokio::time::timeout(
            Duration::from_secs(1),
            connect_quic_candidates(
                Arc::new(TestHost::default()),
                protocol.clone(),
                "192.0.2.100:11012".parse().unwrap(),
                candidates,
                UdpBindOptions::direct_connect(),
                "quic://example.com:11012".parse().unwrap(),
                Duration::from_millis(200),
            ),
        )
        .await
        .expect("queued candidate was not attempted");
        assert!(result.is_ok());
        assert_eq!(protocol.started.load(Ordering::Relaxed), 9);
        assert_eq!(protocol.active.load(Ordering::Relaxed), 0);
    }
}
