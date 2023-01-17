use std::error::Error;
use std::thread;

use crossbeam::channel::unbounded;
use futures::prelude::*;
use libp2p::gossipsub::{
    GossipsubEvent, IdentTopic as Topic,
};
use libp2p::swarm::{Swarm, SwarmEvent};
use libp2p::{identity, Multiaddr, PeerId};
mod behaviour;
mod tss;
use behaviour::main_behaviour::{Behaviour, Event};
use modular_frost::ThresholdKeys;
use modular_frost::curve::Ed25519;

use crate::tss::{KeyGenThreadMessage, KeyGenProcess};



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
    Join(u64),
    // saying that we will rerun the algo and join a new frost schem.
    Create(u64),
    // to say that we joined
    Joined(u64)
}

enum State{
    Joining(u64),
    Joined(u64),
    KeyGenStep1(u64),
    Ready_for_signing(u64)
}
#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public().clone());
    println!("Local peer id: {:?}", local_peer_id);

    let transport = libp2p::development_transport(local_key.clone()).await?;

    let mut behaviour = Behaviour::new(local_key.clone(), local_key.public().clone());
    let mut is_bootstrap_node = false;
    let global_topic = Topic::new("tss_main");

    if let (Some(addr), Some(peerid)) = (std::env::args().nth(1), std::env::args().nth(2)) {
        let remote: Multiaddr = addr.parse()?;
        let peer: PeerId = peerid.parse()?;
        behaviour.kademelia.add_address(&peer, remote);
        behaviour.kademelia.bootstrap().unwrap();
        behaviour.gossipsub.add_explicit_peer(&peer);
        // swarm.dial(remote)?;
        // println!("Dialed {}", addr)
    } else {
        is_bootstrap_node =true;
        behaviour.kademelia.bootstrap();
    }

    let mut swarm = Swarm::with_async_std_executor(transport, behaviour, local_peer_id);
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    // if !is_bootstrap_node{
        swarm.behaviour_mut().gossipsub.subscribe(&global_topic).unwrap();
    // }

    // Dial the peer identified by the multi-address given as the second
    // command-line argument, if any.
    let mut tss: Option<TssScheme> = None;

    let get_tss_members_count = |tss: Option<&TssScheme>|{
        match tss {
            Some(tss) => {
                tss.members.len()
            },
            None => {
                0 // me and the other dude
            },
        }
    };
    
    let mut is_leader = false;

    let should_be_leader = |is_leader_param| {
        if let None = tss{
            true
        }else if is_leader_param{
            true
        }else{
            false
        }
    };



    let join_id: u64 = rand::random();

    let amount_joined = 2;

    let mut state = State::Joining(join_id);
    swarm.behaviour_mut().gossipsub.publish(global_topic.clone(), serde_json::to_string(&TssMessage::Join(join_id)).unwrap())
    .unwrap();
    let mut tss_topic : Option<Topic> = None;

    let (send_channel, receive_channel) = unbounded();
    let (send_result_channel, receive_result_channel) = unbounded();

    let mut key_gen_scope = thread::spawn(||{
        KeyGenProcess(receive_channel, send_result_channel);
    });

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {:?}", address),
            // SwarmEvent::Behaviour(Event::Identify(identity)) => match identity.as_ref() {
            //     libp2p::identify::Event::Received { peer_id, info } => {
            //         if is_bootstrap_node{
            //             continue;
            //         }
            //         // println!(" Received this info: {:?} from peer: {:?}", info, peer_id);
            //     }
            //     _ => {}
            // },
            SwarmEvent::Behaviour(Event::Gossipsub(gossip_event)) =>{
                if is_bootstrap_node{
                    continue;
                }
                match gossip_event {
                    GossipsubEvent::Message { propagation_source, message_id, message } =>{
                        let parsed : TssMessage = serde_json::from_slice(&message.data).unwrap();
                        match &parsed {
                            TssMessage::Join(x) if should_be_leader(is_leader)  => {
                                is_leader = true;
                                state = State::Joining(*x);
                                swarm.behaviour_mut().gossipsub.publish(global_topic.clone(), serde_json::to_string(&TssMessage::Create(join_id)).unwrap())
                                    .unwrap();    
                                tss_topic = Topic::new(format!("tss_session_{}",*x)).into();
                            },
                            //todo, the create message should also have the new member count so we know how much to wait for people.
                            //also use that to calculate id
                            TssMessage::Create(id) => {
                                tss_topic = Topic::new(format!("tss_session_{}",*id)).into();
                                swarm.behaviour_mut().gossipsub.subscribe(tss_topic.as_ref().unwrap()).unwrap();
                                swarm.behaviour_mut().gossipsub.publish(global_topic.clone(), serde_json::to_string(&TssMessage::Joined(join_id)).unwrap())
                                    .unwrap();
                                state = State::Joined(*id);
                            }
                            TssMessage::Joined(id) => {
                                if get_tss_members_count(tss.as_ref()) == 0{
                                    state = State::KeyGenStep1(*id);
                                }
                            }
                            _ => {}
                        }
                        println!("received {:?} from gossipsub from {:?}", parsed, message.source);
                    },
                    _ => {}
                }
            },
            e => {println!("{:?}",e);}
        }
    }
}
