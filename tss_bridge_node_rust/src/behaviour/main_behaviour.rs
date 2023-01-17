use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use libp2p::gossipsub::{Gossipsub, self, ValidationMode, MessageId, GossipsubMessage, MessageAuthenticity, GossipsubEvent};
use libp2p::{identify, autonat, identity, PeerId};
use libp2p::kad::{Kademlia, KademliaEvent};
use libp2p::kad::store::MemoryStore;
use libp2p::swarm::behaviour::toggle::Toggle;
use libp2p::{swarm::NetworkBehaviour};
use libp2p::relay::v2::relay;

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "Event", event_process = false)]
pub struct Behaviour {
    relay:relay::Relay ,
    identify: identify::Behaviour,
    pub kademelia: Kademlia<MemoryStore>,
    pub gossipsub: Gossipsub,
}

impl Behaviour {
    pub fn new(key: identity::Keypair,pub_key: identity::PublicKey) -> Self {
        let mut kademelia = Kademlia::new(PeerId::from_public_key(&pub_key),MemoryStore::new(PeerId::from_public_key(&pub_key),));


        let message_id_fn = |message: &GossipsubMessage| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            MessageId::from(s.finish().to_string())
        };

            // Set a custom gossipsub configuration
        let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
        .duplicate_cache_time(Duration::from_micros(0))
        .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
        .validation_mode(ValidationMode::Permissive) // This sets the kind of message validation. The default is Strict (enforce message signing)
        .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
        .build()
        .expect("Valid config");

    // build a gossipsub network behaviour
    let mut gossipsub = Gossipsub::new(MessageAuthenticity::Signed(key), gossipsub_config)
        .expect("Correct configuration");

        Self {
            relay: relay::Relay::new(PeerId::from(pub_key.clone()), Default::default()),
            identify: identify::Behaviour::new(
                identify::Config::new("rahims_awesome_thing_v1".to_string(), pub_key).with_agent_version(
                    format!("rahims_awesome_thing_v1/{}", env!("CARGO_PKG_VERSION")),
                ),
            ),
            kademelia,
            gossipsub
        }
    }
}



#[derive(Debug)]
pub enum Event {
    Identify(Box<identify::Event>),
    Relay(relay::Event),
    Kademlia(KademliaEvent),
    Autonat(autonat::Event),
    Gossipsub(GossipsubEvent)
}

impl From<identify::Event> for Event {
    fn from(event: identify::Event) -> Self {
        Event::Identify(Box::new(event))
    }
}

impl From<relay::Event> for Event {
    fn from(event: relay::Event) -> Self {
        Event::Relay(event)
    }
}

impl From<KademliaEvent> for Event {
    fn from(event: KademliaEvent) -> Self {
        Event::Kademlia(event)
    }
}

impl From<autonat::Event> for Event {
    fn from(event: autonat::Event) -> Self {
        Event::Autonat(event)
    }
}

impl From<GossipsubEvent> for Event {
    fn from(value: GossipsubEvent) -> Self {
        Event::Gossipsub(value)
    }
}
