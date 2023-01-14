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
    autonat: Toggle<autonat::Behaviour>,
}

impl Behaviour {
    pub fn new(pub_key: identity::PublicKey) -> Self {
        let mut kademelia = Kademlia::new(PeerId::from_public_key(&pub_key),MemoryStore::new(PeerId::from_public_key(&pub_key),));

        let autonat = 
            Some(autonat::Behaviour::new(
                PeerId::from(pub_key.clone()),
                Default::default(),
            ))
        .into();

        Self {
            relay: relay::Relay::new(PeerId::from(pub_key.clone()), Default::default()),
            identify: identify::Behaviour::new(
                identify::Config::new("rahims_awesome_thing_v1".to_string(), pub_key).with_agent_version(
                    format!("rahims_awesome_thing_v1/{}", env!("CARGO_PKG_VERSION")),
                ),
            ),
            kademelia,
            autonat,
        }
    }
}

#[derive(Debug)]
pub enum Event {
    Identify(Box<identify::Event>),
    Relay(relay::Event),
    Kademlia(KademliaEvent),
    Autonat(autonat::Event),
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