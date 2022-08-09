use futures::StreamExt;
use libp2p::core::identity;
use libp2p::core::PeerId;
use libp2p::gossipsub::Gossipsub;
use libp2p::gossipsub::GossipsubEvent;
use libp2p::gossipsub::GossipsubMessage;
use libp2p::gossipsub::IdentTopic;
use libp2p::gossipsub::MessageAuthenticity;
use libp2p::gossipsub::MessageId;
use libp2p::identify::{Identify, IdentifyConfig, IdentifyEvent};
use libp2p::ping::{Ping, PingConfig, PingEvent, PingSuccess};
use libp2p::swarm::{SwarmBuilder, SwarmEvent};
use libp2p::{rendezvous, tokio_development_transport};
use libp2p::{Multiaddr, NetworkBehaviour};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

struct TokioExecutor();

impl libp2p::core::Executor for TokioExecutor {
    fn exec(
        &self,
        future: std::pin::Pin<Box<dyn std::future::Future<Output = ()> + 'static + Send>>,
    ) {
        tokio::task::spawn(future);
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let rendezvous_point_address = "/ip4/127.0.0.1/tcp/62649".parse::<Multiaddr>().unwrap();
    let rendezvous_point = "12D3KooWM94bDWG98w9hawSEipLaSyJjfnFQHQpw4kLMpbg4C3RN"
        .parse()
        .unwrap();

    let identity = identity::Keypair::generate_ed25519();

    let message_id_fn = |message: &GossipsubMessage| {
        let mut s = DefaultHasher::new();
        message.data.hash(&mut s);
        MessageId::from(s.finish().to_string())
    };
    let gossipsub_config = libp2p::gossipsub::GossipsubConfigBuilder::default()
        .message_id_fn(message_id_fn)
        .build()
        .expect("valid gossipsub config");

    let mut gossipsub = Gossipsub::new(
        MessageAuthenticity::Signed(identity.clone()),
        gossipsub_config,
    )
    .expect("valid gossipsub params");

    let topic = IdentTopic::new("_starknet_nodes/SN_GOERLI");

    gossipsub.subscribe(&topic).unwrap();

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
            gossipsub,
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
                if let Err(e) = swarm
                    .behaviour_mut()
                    .gossipsub
                    .publish(topic.clone(), b"deadbeef!".to_owned())
                {
                    log::error!("Failed to publish registration {}", e);
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
    Gossipsub(GossipsubEvent),
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

impl From<GossipsubEvent> for MyEvent {
    fn from(event: GossipsubEvent) -> Self {
        MyEvent::Gossipsub(event)
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(event_process = false)]
#[behaviour(out_event = "MyEvent")]
struct MyBehaviour {
    identify: Identify,
    rendezvous: rendezvous::client::Behaviour,
    ping: Ping,
    gossipsub: Gossipsub,
}
