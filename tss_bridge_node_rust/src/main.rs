use std::error::Error;

use bigint::{U256, U512};
use dkg::ThresholdKeys;
use futures::prelude::*;
use libp2p::gossipsub::{
    Gossipsub, GossipsubEvent, GossipsubMessage, MessageAuthenticity,
    ValidationMode, IdentTopic as Topic,
};
use libp2p::kad::store::MemoryStore;
use libp2p::kad::Kademlia;
use libp2p::ping::Ping;
use libp2p::swarm::{dial_opts::DialOpts, Swarm, SwarmEvent};
use libp2p::{identity, ping, Multiaddr, PeerId};
mod behaviour;
use behaviour::main_behaviour::{Behaviour, Event};
use modular_frost::curve::Ed25519;



struct TssMember {
    stellar_key: String,
    index: u16,
}
struct TssScheme {
    members: Vec<TssMember>,
    thresholdkeys: ThresholdKeys<Ed25519>,
}
#[derive(serde::Deserialize,serde::Serialize,Debug)]
enum TssMessage{
    Join
}
#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public().clone());
    println!("Local peer id: {:?}", local_peer_id);

    let transport = libp2p::development_transport(local_key.clone()).await?;

    let mut behaviour = Behaviour::new(local_key.clone(), local_key.public().clone());
    let mut is_bootstrap_node = false;
    let topic = Topic::new("find_tss");

    if let (Some(addr), Some(peerid)) = (std::env::args().nth(1), std::env::args().nth(2)) {
        let remote: Multiaddr = addr.parse()?;
        let peer: PeerId = peerid.parse()?;
        behaviour.kademelia.add_address(&peer, remote);
        behaviour.kademelia.bootstrap();
        behaviour.gossipsub.subscribe(&topic).unwrap();
        // swarm.dial(remote)?;
        // println!("Dialed {}", addr)
    } else {
        is_bootstrap_node =true;
        behaviour.kademelia.bootstrap();
    }

    let mut swarm = Swarm::with_async_std_executor(transport, behaviour, local_peer_id);
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // Dial the peer identified by the multi-address given as the second
    // command-line argument, if any.
    let mut tss: Option<TssScheme> = None;



    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {:?}", address),
            SwarmEvent::Behaviour(Event::Identify(identity)) => match identity.as_ref() {
                libp2p::identify::Event::Received { peer_id, info } => {
                    if is_bootstrap_node{
                        continue;
                    }
                    println!(" Received this info: {:?} from peer: {:?}", info, peer_id);
                    if let None = tss{
                        if U512::from_big_endian(&peer_id.to_bytes()[..]) > U512::from_big_endian(&local_peer_id.to_bytes()[..]){
                            swarm.behaviour_mut().gossipsub.publish(topic.clone(), serde_json::to_string(&TssMessage::Join).unwrap())
                            .unwrap();
                        }

                    }
                }
                _ => {}
            },
            SwarmEvent::Behaviour(Event::Gossipsub(gossip_event)) =>{
                if is_bootstrap_node{
                    continue;
                }
                match gossip_event {
                    GossipsubEvent::Message { propagation_source, message_id, message } =>{
                        let parsed : TssMessage = serde_json::from_slice(&message.data).unwrap();
                        println!("receiced {:?} from gossipsub from {:?}", parsed, message.source);
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }
}
