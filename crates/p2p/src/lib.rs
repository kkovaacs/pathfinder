#![deny(rust_2018_idioms)]

use std::time::Duration;

use futures::StreamExt;
use libp2p::gossipsub::IdentTopic;
use libp2p::identify::{IdentifyEvent, IdentifyInfo};
use libp2p::identity::Keypair;
use libp2p::kad;
use libp2p::swarm::{SwarmBuilder, SwarmEvent};
use libp2p::Multiaddr;
use tokio::task::JoinHandle;
use tracing::Instrument;

mod behaviour;
mod executor;
mod transport;

/// Starts a P2P task that drives libp2p communication.
#[tracing::instrument(name = "p2p", skip_all)]
pub fn start(
    keypair: Keypair,
    listen_on: Multiaddr,
    bootstrap_addresses: Vec<Multiaddr>,
) -> anyhow::Result<JoinHandle<()>> {
    let peer_id = keypair.public().to_peer_id();

    let mut swarm = SwarmBuilder::new(
        transport::create(&keypair),
        behaviour::Behaviour::new(&keypair),
        peer_id,
    )
    .executor(Box::new(executor::TokioExecutor()))
    .build();

    swarm.listen_on(listen_on)?;

    for bootstrap_address in bootstrap_addresses {
        swarm.dial(bootstrap_address)?;
    }

    let topic = IdentTopic::new("_starknet_nodes/SN_GOERLI");
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    let join_handle = tokio::task::spawn(
        async move {
            main_loop(swarm).await;
        }
        .in_current_span(),
    );
    Ok(join_handle)
}

async fn main_loop(mut swarm: libp2p::swarm::Swarm<behaviour::Behaviour>) {
    const BOOTSTRAP_INTERVAL: Duration = Duration::from_secs(30);
    // delay bootstrap so that by the time we attempt it we've connected to the bootstrap node
    let bootstrap_start = tokio::time::Instant::now() + Duration::from_secs(5);
    let mut bootstrap_interval = tokio::time::interval_at(bootstrap_start, BOOTSTRAP_INTERVAL);

    loop {
        let bootstrap_interval_tick = bootstrap_interval.tick();
        tokio::pin!(bootstrap_interval_tick);

        tokio::select! {
            _ = bootstrap_interval_tick => {
                tracing::debug!("Doing periodical bootstrap");
                _ = swarm.behaviour_mut().kademlia.bootstrap();

                let network_info = swarm.network_info();
                let num_peers = network_info.num_peers();
                let connection_counters = network_info.connection_counters();
                let num_connections = connection_counters.num_connections();
                tracing::info!(%num_peers, %num_connections, "Peer-to-peer status")
            }
            Some(event) = swarm.next() => {
                match event {
                    SwarmEvent::Behaviour(behaviour::Event::Identify(e)) => {
                        if let IdentifyEvent::Received {
                            peer_id,
                            info:
                                IdentifyInfo {
                                    listen_addrs,
                                    protocols,
                                    ..
                                },
                        } = *e
                        {
                            if protocols
                                .iter()
                                .any(|p| p.as_bytes() == kad::protocol::DEFAULT_PROTO_NAME)
                            {
                                for addr in listen_addrs {
                                    swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                                }
                                tracing::debug!(%peer_id, "Added peer to DHT");
                            }
                        }
                    }
                    SwarmEvent::Behaviour(behaviour::Event::Gossipsub(e)) => {
                        tracing::info!(?e, "Gossipsub event");
                    }
                    event => {
                        tracing::debug!(?event, "Ignoring event");
                    }
                }
            }
        }
    }
}