use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::task::Poll;

use libp2p::core::PeerRecord;
use libp2p::gossipsub::error::SubscriptionError;
use libp2p::gossipsub::{
    Gossipsub, GossipsubEvent, GossipsubMessage, IdentTopic, MessageAuthenticity, MessageId,
};
use libp2p::identity::Keypair;
use libp2p::swarm::{ConnectionHandler, NetworkBehaviour, NetworkBehaviourAction};

#[derive(Debug)]
pub enum PubsubEvent {
    Gossipsub(GossipsubEvent),
    Discovery(NewNode),
}

#[derive(Debug, Clone)]
pub struct Capability(String);

#[derive(Debug, Clone)]
pub struct NewNode {
    pub peer: PeerRecord,
}

pub struct Pubsub {
    gossipsub: Gossipsub,
    discovered_nodes: Vec<NewNode>,
}

impl Pubsub {
    pub fn new(identity: Keypair) -> Self {
        let message_id_fn = |message: &GossipsubMessage| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            MessageId::from(s.finish().to_string())
        };
        let gossipsub_config = libp2p::gossipsub::GossipsubConfigBuilder::default()
            .message_id_fn(message_id_fn)
            .build()
            .expect("valid gossipsub config");

        let gossipsub = Gossipsub::new(
            MessageAuthenticity::Signed(identity.clone()),
            gossipsub_config,
        )
        .expect("valid gossipsub params");

        Self {
            gossipsub,
            discovered_nodes: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, topic: &IdentTopic) -> Result<bool, SubscriptionError> {
        self.gossipsub.subscribe(topic)
    }
}

impl NetworkBehaviour for Pubsub {
    type ConnectionHandler = <Gossipsub as NetworkBehaviour>::ConnectionHandler;
    type OutEvent = PubsubEvent;

    fn new_handler(&mut self) -> Self::ConnectionHandler {
        self.gossipsub.new_handler()
    }

    fn addresses_of_peer(&mut self, _: &libp2p::PeerId) -> Vec<libp2p::Multiaddr> {
        // FIXME: we should look up peers discovered through gossipsub messages
        vec![]
    }

    fn inject_event(
        &mut self,
        peer_id: libp2p::PeerId,
        connection: libp2p::core::connection::ConnectionId,
        event: <<Self::ConnectionHandler as libp2p::swarm::IntoConnectionHandler>::Handler as ConnectionHandler>::OutEvent,
    ) {
        // FIXME: this should be modified to use our own event type
        self.gossipsub.inject_event(peer_id, connection, event)
    }

    fn inject_connection_established(
        &mut self,
        peer_id: &libp2p::PeerId,
        connection_id: &libp2p::core::connection::ConnectionId,
        endpoint: &libp2p::core::ConnectedPoint,
        failed_addresses: Option<&Vec<libp2p::Multiaddr>>,
        other_established: usize,
    ) {
        self.gossipsub.inject_connection_established(
            peer_id,
            connection_id,
            endpoint,
            failed_addresses,
            other_established,
        )
    }

    fn inject_connection_closed(
        &mut self,
        peer_id: &libp2p::PeerId,
        connection_id: &libp2p::core::connection::ConnectionId,
        endpoint: &libp2p::core::ConnectedPoint,
        handler: <Self::ConnectionHandler as libp2p::swarm::IntoConnectionHandler>::Handler,
        remaining_established: usize,
    ) {
        self.gossipsub.inject_connection_closed(
            peer_id,
            connection_id,
            endpoint,
            handler,
            remaining_established,
        )
    }

    fn inject_address_change(
        &mut self,
        peer_id: &libp2p::PeerId,
        connection_id: &libp2p::core::connection::ConnectionId,
        old: &libp2p::core::ConnectedPoint,
        new: &libp2p::core::ConnectedPoint,
    ) {
        self.gossipsub
            .inject_address_change(peer_id, connection_id, old, new)
    }

    fn inject_dial_failure(
        &mut self,
        peer_id: Option<libp2p::PeerId>,
        handler: Self::ConnectionHandler,
        error: &libp2p::swarm::DialError,
    ) {
        self.gossipsub.inject_dial_failure(peer_id, handler, error)
    }

    fn inject_listen_failure(
        &mut self,
        local_addr: &libp2p::Multiaddr,
        send_back_addr: &libp2p::Multiaddr,
        handler: Self::ConnectionHandler,
    ) {
        self.gossipsub
            .inject_listen_failure(local_addr, send_back_addr, handler)
    }

    fn inject_new_listener(&mut self, id: libp2p::core::transport::ListenerId) {
        self.gossipsub.inject_new_listener(id)
    }

    fn inject_new_listen_addr(
        &mut self,
        id: libp2p::core::transport::ListenerId,
        addr: &libp2p::Multiaddr,
    ) {
        self.gossipsub.inject_new_listen_addr(id, addr)
    }

    fn inject_expired_listen_addr(
        &mut self,
        id: libp2p::core::transport::ListenerId,
        addr: &libp2p::Multiaddr,
    ) {
        self.gossipsub.inject_expired_listen_addr(id, addr)
    }

    fn inject_listener_error(
        &mut self,
        id: libp2p::core::transport::ListenerId,
        err: &(dyn std::error::Error + 'static),
    ) {
        self.gossipsub.inject_listener_error(id, err)
    }

    fn inject_listener_closed(
        &mut self,
        id: libp2p::core::transport::ListenerId,
        reason: Result<(), &std::io::Error>,
    ) {
        self.gossipsub.inject_listener_closed(id, reason)
    }

    fn inject_new_external_addr(&mut self, addr: &libp2p::Multiaddr) {
        self.gossipsub.inject_new_external_addr(addr)
    }

    fn inject_expired_external_addr(&mut self, addr: &libp2p::Multiaddr) {
        self.gossipsub.inject_expired_external_addr(addr)
    }

    fn poll(
        &mut self,
        cx: &mut std::task::Context<'_>,
        params: &mut impl libp2p::swarm::PollParameters,
    ) -> std::task::Poll<NetworkBehaviourAction<Self::OutEvent, Self::ConnectionHandler>> {
        if let Some(new_node) = self.discovered_nodes.pop() {
            return Poll::Ready(NetworkBehaviourAction::GenerateEvent(
                PubsubEvent::Discovery(new_node),
            ));
        }

        match futures::ready!(self.gossipsub.poll(cx, params)) {
            NetworkBehaviourAction::GenerateEvent(GossipsubEvent::Message {
                propagation_source,
                message_id,
                message,
            }) => {
                log::info!(
                    "Received gossipsub message {:?} from {:?} id {:?}",
                    message,
                    propagation_source,
                    message_id
                );

                match wire::Message::from_protobuf_encoding(&message.data) {
                    Ok(message) => match message {
                        wire::Message::NewNode(new_node) => self.discovered_nodes.push(new_node),
                    },
                    Err(e) => {
                        log::error!("Ignoring invalid discovery message {}", e);
                    }
                }

                Poll::Ready(NetworkBehaviourAction::GenerateEvent(
                    PubsubEvent::Gossipsub(GossipsubEvent::Message {
                        propagation_source,
                        message_id,
                        message,
                    }),
                ))
            }
            NetworkBehaviourAction::GenerateEvent(event) => {
                return Poll::Ready(NetworkBehaviourAction::GenerateEvent(
                    PubsubEvent::Gossipsub(event),
                ))
            }
            NetworkBehaviourAction::Dial { opts, handler } => {
                Poll::Ready(NetworkBehaviourAction::Dial { opts, handler })
            }
            NetworkBehaviourAction::NotifyHandler {
                peer_id,
                handler,
                event,
            } => Poll::Ready(NetworkBehaviourAction::NotifyHandler {
                peer_id,
                handler,
                event,
            }),
            NetworkBehaviourAction::ReportObservedAddr { address, score } => {
                Poll::Ready(NetworkBehaviourAction::ReportObservedAddr { address, score })
            }
            NetworkBehaviourAction::CloseConnection {
                peer_id,
                connection,
            } => Poll::Ready(NetworkBehaviourAction::CloseConnection {
                peer_id,
                connection,
            }),
        }
    }
}

mod wire {
    use std::fmt;

    use libp2p::{
        core::{peer_record, signed_envelope},
        identity::Keypair,
        Multiaddr,
    };

    #[derive(Debug)]
    pub enum Error {
        InvalidMessage(prost::DecodeError),
        EnvelopeError(signed_envelope::DecodingError),
        PeerRecordError(peer_record::FromEnvelopeError),
    }

    impl From<prost::DecodeError> for Error {
        fn from(e: prost::DecodeError) -> Self {
            Self::InvalidMessage(e)
        }
    }

    impl From<signed_envelope::DecodingError> for Error {
        fn from(e: signed_envelope::DecodingError) -> Self {
            Self::EnvelopeError(e)
        }
    }

    impl From<peer_record::FromEnvelopeError> for Error {
        fn from(e: peer_record::FromEnvelopeError) -> Self {
            Self::PeerRecordError(e)
        }
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::InvalidMessage(_) => write!(f, "Failed to decode message"),
                Self::EnvelopeError(_) => write!(f, "Failed to decode envelope"),
                Self::PeerRecordError(_) => write!(f, "Failed to decode peer record"),
            }
        }
    }

    impl std::error::Error for Error {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                Self::InvalidMessage(inner) => Some(inner),
                Self::EnvelopeError(inner) => Some(inner),
                Self::PeerRecordError(inner) => Some(inner),
            }
        }
    }

    pub enum Message {
        NewNode(super::NewNode),
    }

    impl Message {
        pub fn from_protobuf_encoding(bytes: &[u8]) -> Result<Self, Error> {
            use prost::Message;

            let message = crate::proto::NewNode::decode(bytes)?;
            let envelope = signed_envelope::SignedEnvelope::from_protobuf_encoding(
                &message.signed_peer_record,
            )?;
            let peer_record = peer_record::PeerRecord::from_signed_envelope(envelope)?;

            Ok(Self::NewNode(super::NewNode { peer: peer_record }))
        }

        pub fn new(key: &Keypair, addresses: Vec<Multiaddr>) -> anyhow::Result<Self> {
            let record = peer_record::PeerRecord::new(key, addresses)?;
            Ok(Self::NewNode(super::NewNode { peer: record }))
        }

        pub fn into_protobuf_encoding(self) -> Vec<u8> {
            match self {
                Self::NewNode(new_node) => new_node
                    .peer
                    .into_signed_envelope()
                    .into_protobuf_encoding(),
            }
        }
    }
}
