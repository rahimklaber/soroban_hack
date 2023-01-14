#![no_std]


use soroban_auth::{Identifier, Signature};
use soroban_sdk::{
    contractimpl, contracttype, Env, BytesN, contracterror, panic_with_error
};
mod dao_token;
mod token {
    soroban_sdk::contractimport!(file = "./soroban_token_spec.wasm");
}

// #[contracttype]
// #[derive(Clone, Debug)]
// pub struct ProposalVote {
//     pub voter: Address,
//     pub prop_id: u32,
// }

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    // power of 
    PowerOf(Identifier),
    DelegatTo(Identifier),
    CanMint(BytesN<32>),

}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    CannotMint = 1,
    InsufficientPower = 2,
}



pub trait DaoBridgeTrait {
    fn init(env: Env, token_id: BytesN<32>);
    // mint without creating proposal. 
    // The [signature] should have enough power to mint.
    fn fast_mint(env: Env, reference: BytesN<32>, recipient: Identifier, amount: i128, token_id: BytesN<32>, signature: Signature, nonce: i128);
}

pub struct DaoBridgeContract;

#[contractimpl]
impl DaoBridgeTrait for DaoBridgeContract {
    fn init(env: Env, token_id: BytesN<32>){
        approve_token_minting(&env, token_id);
        
    }

    fn fast_mint(env: Env, reference: BytesN<32>, recipient: Identifier, amount: i128, token_id: BytesN<32>, signature: Signature, nonce: i128){
        if !can_mint(&env, token_id.clone()){
            panic_with_error!(&env,Error::CannotMint);
        }

        if !is_enough_power(&env, &signature){
            panic_with_error!(&env, Error::InsufficientPower);
        }

        mint(&env, recipient, amount, token_id);
    }

}

fn approve_token_minting(env: &Env, token_id: BytesN<32>){
    env
    .storage()
    .set(DataKey::CanMint(token_id), true);
}

// mint some tokens for recipient;
fn mint(env: &Env, recipient: Identifier, amount: i128, token_id: BytesN<32>){
    let token_client = token::Client::new(&env, token_id);
    let nonce = token_client.nonce(&Identifier::Contract(env.current_contract()));
    token_client.mint(&Signature::Invoker, &nonce, &recipient, &amount)
}

// checks if there is enough power to mint
// in other words, checks if voting power / total power > 0.5
fn is_enough_power(env: &Env, signature: &Signature) -> bool{
    //todo
    power_of(env, signature) > 0
}

fn power_of(env: &Env, signature: &Signature) -> i128{
    //todo
    return 1;
}

// whether the bridge can mint this token.
fn can_mint(env: &Env, token_id: BytesN<32>) -> bool{
    env
    .storage()
    .get(DataKey::CanMint(token_id))
    .unwrap_or(Ok(false))
    .unwrap()
}

// #[cfg(test)]
// mod test;