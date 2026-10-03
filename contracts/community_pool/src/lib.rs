#![no_std]

use soroban_sdk::{contract, contractimpl};

// The reveal entrypoint will use this in Step 3; keep hash inputs off public calls.
#[allow(dead_code)]
mod commitment;

/// Scaffold only. Pool, auction, and repayment logic are not implemented yet.
#[contract]
pub struct CommunityPool;

#[contractimpl]
impl CommunityPool {
    pub fn version() -> u32 {
        1
    }
}
