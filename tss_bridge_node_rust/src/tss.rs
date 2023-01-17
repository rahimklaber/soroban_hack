use core::panic;
use std::collections::HashMap;

use crossbeam::channel::{Receiver, Sender};
use modular_frost::{curve::{Ed25519, Ciphersuite}, ThresholdParams, dkg::{frost::{KeyGenMachine, Commitments, KeyMachine, SecretShare}, encryption::{EncryptionKeyMessage, EncryptedMessage}}, ThresholdCore};
use rand::rngs::OsRng;

pub enum KeyGenThreadMessage  {
    Start(u16,u16,u16),
    // commitment from another user
    Commitment((u16,EncryptionKeyMessage<Ed25519,Commitments<Ed25519>>)),
    // share from another user.
    SecretShare((u16,EncryptedMessage<Ed25519,SecretShare<<modular_frost::curve::Ed25519 as Ciphersuite>::F>>))
}   

pub enum KeyGenThreadOutput {
    // out commitments for others
    Commitments(EncryptionKeyMessage<Ed25519, Commitments<Ed25519>>),
    // out shares for others.
    SecretShares(HashMap<u16,EncryptedMessage<Ed25519,SecretShare<<modular_frost::curve::Ed25519 as Ciphersuite>::F>>>),
    Key(ThresholdCore<Ed25519>)
}

pub fn KeyGenProcess(receive_channel: Receiver<KeyGenThreadMessage>,
    send_channel: Sender<KeyGenThreadOutput>
){
    let mut start_msg = receive_channel.recv().unwrap();
    let (threshold, n, index) = match start_msg {
        KeyGenThreadMessage::Start(t ,n, i) => {(t,n,i)},
        _ => {panic!("invalid msg")}
    };
    
    let key_params = ThresholdParams::new(threshold,n,index ).unwrap();

    let key_gen_machine = KeyGenMachine::<Ed25519>::new(key_params.clone(),"yeet".into());

    let (secret_share_machine, commitments) = key_gen_machine.generate_coefficients(&mut OsRng);

    send_channel.send(KeyGenThreadOutput::Commitments(commitments));

    let mut commitments_map = HashMap::<u16,EncryptionKeyMessage<Ed25519,Commitments<Ed25519>>>::new();
    loop {
        let (index, commitment) = match receive_channel.recv().unwrap() {
            KeyGenThreadMessage::Commitment((index,commitment)) => {
                (index,commitment)
            },
            _ => panic!("invalid msg")
        };
        commitments_map.insert(index, commitment);
        if commitments_map.len() == (n - 1) as usize{
            break;
        }
    }
    let (key_machine, shares) = secret_share_machine.generate_secret_shares(&mut OsRng, commitments_map).unwrap();

    send_channel.send(KeyGenThreadOutput::SecretShares(shares));

    let mut shares_map = HashMap::<u16,EncryptedMessage<_,_>>::new();

    loop {
        let (index, share) = match receive_channel.recv().unwrap(){
            KeyGenThreadMessage::SecretShare((index,share)) => (index,share),
            _ => {panic!("invalid msg")}
        };
        shares_map.insert(index, share);
        if shares_map.len() == (n -1) as usize{
            break;
        }
    }
    
    let blame_machine = key_machine.calculate_share(&mut OsRng, shares).unwrap();
    let key = blame_machine.complete();
    send_channel.send(KeyGenThreadOutput::Key(key));
    
}