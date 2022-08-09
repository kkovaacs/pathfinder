use futures::StreamExt;
use libp2p::core::identity;
use libp2p::core::PeerId;
use libp2p::gossipsub::IdentTopic;
use libp2p::identify::Identify;
use libp2p::identify::IdentifyConfig;
use libp2p::identify::IdentifyEvent;
use libp2p::identity::ed25519;
use libp2p::ping;
use libp2p::ping::{Ping, PingEvent};
use libp2p::swarm::{SwarmBuilder, SwarmEvent};
use libp2p::NetworkBehaviour;
use libp2p::{rendezvous, tokio_development_transport};

use p2p_discovery::{pubsub, TokioExecutor};

/// Examples for the rendezvous protocol:
///
/// 1. Run the rendezvous server:
///    RUST_LOG=info cargo run --example rendezvous_point
/// 2. Register a peer:
///    RUST_LOG=info cargo run --example register_with_identify
/// 3. Try to discover the peer from (2):
///    RUST_LOG=info cargo run --example discover
#[tokio::main]
async fn main() {
    env_logger::init();

    let identity = identity::Keypair::Ed25519(ed25519::Keypair::generate());

    let mut swarm = {
        let mut pubsub = pubsub::Pubsub::new(identity.clone());
        let topic = IdentTopic::new("_starknet_nodes/SN_GOERLI");
        pubsub.subscribe(&topic).unwrap();

        SwarmBuilder::new(
            tokio_development_transport(identity.clone()).unwrap(),
            MyBehaviour {
                identify: Identify::new(IdentifyConfig::new(
                    "starknet/0.1.0".to_string(),
                    identity.public(),
                )),
                rendezvous: rendezvous::server::Behaviour::new(
                    rendezvous::server::Config::default(),
                ),
                ping: Ping::new(ping::Config::new().with_keep_alive(true)),
                pubsub,
            },
            PeerId::from(identity.public()),
        )
        .executor(Box::new(TokioExecutor()))
        .build()
    };

    log::info!("Local peer id: {}", swarm.local_peer_id());

    swarm
        .listen_on("/ip4/0.0.0.0/tcp/62649".parse().unwrap())
        .unwrap();

    while let Some(event) = swarm.next().await {
        match event {
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                log::info!("Connected to {}", peer_id);
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                log::info!("Disconnected from {}", peer_id);
            }
            SwarmEvent::Behaviour(MyEvent::Rendezvous(
                rendezvous::server::Event::PeerRegistered { peer, registration },
            )) => {
                log::info!(
                    "Peer {} registered for namespace '{}'",
                    peer,
                    registration.namespace
                );
            }
            SwarmEvent::Behaviour(MyEvent::Rendezvous(
                rendezvous::server::Event::DiscoverServed {
                    enquirer,
                    registrations,
                },
            )) => {
                log::info!(
                    "Served peer {} with {} registrations",
                    enquirer,
                    registrations.len()
                );
            }
            SwarmEvent::Behaviour(MyEvent::Pubsub(pubsub::PubsubEvent::Discovery(new_node))) => {
                log::info!("Discovered peer on pubsub {:?}", new_node);
            }
            other => {
                log::debug!("Unhandled {:?}", other);
            }
        }
    }
}

#[derive(Debug)]
enum MyEvent {
    Rendezvous(rendezvous::server::Event),
    Ping(PingEvent),
    Identify(IdentifyEvent),
    Pubsub(pubsub::PubsubEvent),
}

impl From<rendezvous::server::Event> for MyEvent {
    fn from(event: rendezvous::server::Event) -> Self {
        MyEvent::Rendezvous(event)
    }
}

impl From<PingEvent> for MyEvent {
    fn from(event: PingEvent) -> Self {
        MyEvent::Ping(event)
    }
}

impl From<IdentifyEvent> for MyEvent {
    fn from(event: IdentifyEvent) -> Self {
        MyEvent::Identify(event)
    }
}

impl From<pubsub::PubsubEvent> for MyEvent {
    fn from(event: pubsub::PubsubEvent) -> Self {
        MyEvent::Pubsub(event)
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(event_process = false)]
#[behaviour(out_event = "MyEvent")]
struct MyBehaviour {
    identify: Identify,
    rendezvous: rendezvous::server::Behaviour,
    ping: Ping,
    pubsub: pubsub::Pubsub,
}
