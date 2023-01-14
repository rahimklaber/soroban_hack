use dkg::ThresholdKeys;
use libp2p::swarm::NetworkBehaviour;
use modular_frost::curve::Ed25519;



pub struct Config{
    keys: ThresholdKeys<Ed25519>
}
pub struct TssBehaviour{
    config: Config
}

// impl NetworkBehaviour for TssBehaviour{
//     type ConnectionHandler;

//     type OutEvent;

//     fn new_handler(&mut self) -> Self::ConnectionHandler {
//         todo!()
//     }

//     fn poll(
//         &mut self,
//         cx: &mut std::task::Context<'_>,
//         params: &mut impl libp2p::swarm::PollParameters,
//     ) -> std::task::Poll<libp2p::swarm::NetworkBehaviourAction<Self::OutEvent, Self::ConnectionHandler>> {
//         todo!()
//     }
// }