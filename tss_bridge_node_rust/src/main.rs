use std::error::Error;

use futures::prelude::*;
use libp2p::kad::Kademlia;
use libp2p::kad::store::MemoryStore;
use libp2p::swarm::{Swarm, SwarmEvent, dial_opts::DialOpts};
use libp2p::{identity, Multiaddr, PeerId, ping};
use behaviour::Behaviour;
mod behaviour;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public().clone());
    println!("Local peer id: {:?}", local_peer_id);

    let transport = libp2p::development_transport(local_key.clone()).await?;

  
    let mut behaviour = Behaviour::new(local_key.public().clone());
    if let (Some(addr), Some(peerid)) =( std::env::args().nth(1),  std::env::args().nth(2)) {
        let remote: Multiaddr = addr.parse()?;
        let peer: PeerId = peerid.parse()?;
        behaviour.kademelia.add_address(&peer, remote);
        behaviour.kademelia.bootstrap();

        // swarm.dial(remote)?;
        // println!("Dialed {}", addr)
    }else{
        behaviour.kademelia.bootstrap();
    }
    let mut swarm = Swarm::with_async_std_executor(transport, behaviour, local_peer_id);
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // Dial the peer identified by the multi-address given as the second
    // command-line argument, if any.

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {:?}", address),
            SwarmEvent::Behaviour(event) => println!("{:?}", event),
            _ => {}
        }
    }

}