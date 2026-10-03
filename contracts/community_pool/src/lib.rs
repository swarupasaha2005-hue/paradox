#![no_std]

use soroban_sdk::{contract, contractimpl};

/// Scaffold only. Pool, auction, and repayment logic are not implemented yet.
#[contract]
pub struct CommunityPool;

#[contractimpl]
impl CommunityPool {
    pub fn version() -> u32 {
        1
    }
}
