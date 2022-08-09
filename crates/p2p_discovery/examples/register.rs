use std::time::Duration;

use futures::StreamExt;
use libp2p::core::identity;
use libp2p::core::PeerId;
use libp2p::gossipsub::IdentTopic;
use libp2p::identify::{Identify, IdentifyConfig, IdentifyEvent};
use libp2p::ping::{Ping, PingConfig, PingEvent, PingSuccess};
use libp2p::swarm::{SwarmBuilder, SwarmEvent};
use libp2p::{rendezvous, tokio_development_transport};
use libp2p::{Multiaddr, NetworkBehaviour};

use p2p_discovery::{pubsub, TokioExecutor};

#[tokio::main]
async fn main() {
    env_logger::init();

    let rendezvous_point_address = "/ip4/127.0.0.1/tcp/62649".parse::<Multiaddr>().unwrap();
    let rendezvous_point = "12D3KooWJjJUJ2vUx8iwnX5oLbCurKEw9TE9BKoqAbVF2J9iKxur"
        .parse()
        .unwrap();

    let identity = identity::Keypair::generate_ed25519();

    let topic = IdentTopic::new("_starknet_nodes/SN_GOERLI");
    let pubsub = pubsub::Pubsub::new(identity.clone(), topic).unwrap();

    let mut swarm = SwarmBuilder::new(
        tokio_development_transport(identity.clone()).unwrap(),
        MyBehaviour {
            identify: Identify::new(IdentifyConfig::new(
                "starknet/0.1.0".to_string(),
                identity.public(),
            )),
            rendezvous: rendezvous::client::Behaviour::new(identity.clone()),
            ping: Ping::new(
                PingConfig::new()
                    .with_interval(Duration::from_secs(1))
                    .with_keep_alive(true),
            ),
            pubsub,
        },
        PeerId::from(identity.public()),
    )
    .executor(Box::new(TokioExecutor()))
    .build();

    log::info!("Local peer id: {}", swarm.local_peer_id());

    let _ = swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap());

    swarm.dial(rendezvous_point_address).unwrap();

    while let Some(event) = swarm.next().await {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                log::info!("Listening on {}", address);
            }
            SwarmEvent::ConnectionClosed {
                peer_id,
                cause: Some(error),
                ..
            } if peer_id == rendezvous_point => {
                log::error!("Lost connection to rendezvous point {}", error);
            }
            // once `/identify` did its job, we know our external address and can register
            SwarmEvent::Behaviour(MyEvent::Identify(IdentifyEvent::Received { .. })) => {
                swarm.behaviour_mut().rendezvous.register(
                    rendezvous::Namespace::from_static("_starknet_discover/SN_GOERLI"),
                    rendezvous_point,
                    None,
                );

                let external_addresses = swarm
                    .external_addresses()
                    .map(|r| r.addr.clone())
                    .collect::<Vec<Multiaddr>>();

                if let Err(e) = swarm
                    .behaviour_mut()
                    .pubsub
                    .register_node(&identity, external_addresses)
                {
                    log::error!("Failed to publish registration via pubsub {}", e);
                }
            }
            SwarmEvent::Behaviour(MyEvent::Rendezvous(rendezvous::client::Event::Registered {
                namespace,
                ttl,
                rendezvous_node,
            })) => {
                log::info!(
                    "Registered for namespace '{}' at rendezvous point {} for the next {} seconds",
                    namespace,
                    rendezvous_node,
                    ttl
                );
            }
            SwarmEvent::Behaviour(MyEvent::Rendezvous(
                rendezvous::client::Event::RegisterFailed(error),
            )) => {
                log::error!("Failed to register {}", error);
                return;
            }
            SwarmEvent::Behaviour(MyEvent::Ping(PingEvent {
                peer,
                result: Ok(PingSuccess::Ping { rtt }),
            })) if peer != rendezvous_point => {
                log::info!("Ping to {} is {}ms", peer, rtt.as_millis())
            }
            other => {
                log::debug!("Unhandled {:?}", other);
            }
        }
    }
}

#[derive(Debug)]
enum MyEvent {
    Rendezvous(rendezvous::client::Event),
    Identify(IdentifyEvent),
    Ping(PingEvent),
    Pubsub(pubsub::PubsubEvent),
}

impl From<rendezvous::client::Event> for MyEvent {
    fn from(event: rendezvous::client::Event) -> Self {
        MyEvent::Rendezvous(event)
    }
}

impl From<IdentifyEvent> for MyEvent {
    fn from(event: IdentifyEvent) -> Self {
        MyEvent::Identify(event)
    }
}

impl From<PingEvent> for MyEvent {
    fn from(event: PingEvent) -> Self {
        MyEvent::Ping(event)
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
    rendezvous: rendezvous::client::Behaviour,
    ping: Ping,
    pubsub: pubsub::Pubsub,
}
