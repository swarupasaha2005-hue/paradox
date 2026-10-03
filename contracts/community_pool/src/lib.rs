#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, token::TokenClient,
    Address, BytesN, Env, MuxedAddress, Vec,
};

// Hash inputs stay private until reveal; the preimage is never stored.
mod commitment;

const MAX_ROUND_REQUESTS: u32 = 10;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    MemberAlreadyExists = 4,
    MemberNotFound = 5,
    InactiveMember = 6,
    InvalidAmount = 7,
    NotEligible = 8,
    RequestNotFound = 9,
    InvalidRequestStatus = 10,
    RoundNotFound = 11,
    InvalidDeadline = 12,
    InsufficientPoolBalance = 13,
    InvalidAsset = 14,
    Overflow = 15,
    InvalidRoundSize = 16,
    DuplicateParticipant = 17,
    NotParticipant = 18,
    InvalidRoundStatus = 19,
    CommitClosed = 20,
    RevealNotOpen = 21,
    RevealClosed = 22,
    CommitmentAlreadyExists = 23,
    CommitmentNotFound = 24,
    CommitmentMismatch = 25,
    RevealAlreadyExists = 26,
    InvalidBid = 27,
    FinalizationTooEarly = 28,
    NoWinner = 29,
    AlreadySettled = 30,
    InvalidRepayment = 31,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Community {
    pub initialized: bool,
    pub admin: Address,
    pub asset: Address,
    pub contribution_amount: i128,
    pub minimum_contributions: u32,
    pub financing_limit: i128,
    pub member_count: u32,
    pub available_pool: i128,
    pub request_counter: u64,
    pub round_counter: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Member {
    pub active: bool,
    pub contributions_completed: u32,
    pub contributions_missed: u32,
    pub total_contributed: i128,
    pub financing_rounds_completed: u32,
    pub repayments_completed: u32,
    pub repayments_missed: u32,
    pub defaults: u32,
    pub unresolved_default: bool,
    pub outstanding_financing: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EligibilityResult {
    pub eligible: bool,
    pub is_member: bool,
    pub active: bool,
    pub contribution_requirement_met: bool,
    pub no_unresolved_default: bool,
    pub no_outstanding_financing: bool,
    pub valid_amount: bool,
    pub within_financing_limit: bool,
    pub contributions_completed: u32,
    pub minimum_contributions: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RequestStatus {
    Pending,
    IncludedInRound,
    Funded,
    Repaid,
    Defaulted,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapitalRequest {
    pub id: u64,
    pub member: Address,
    pub maximum_amount: i128,
    pub purpose_hash: BytesN<32>,
    pub status: RequestStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RoundStatus {
    Commit,
    Reveal,
    Finalized,
    Settled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Round {
    pub id: u64,
    pub request_ids: Vec<u64>,
    pub participants: Vec<Address>,
    pub available_capital: i128,
    pub commit_deadline: u64,
    pub reveal_deadline: u64,
    pub status: RoundStatus,
    pub winner: Option<Address>,
    pub winning_bid: Option<i128>,
    pub settled: bool,
    pub valid_reveal_count: u32,
    pub outstanding_amount: i128,
    pub due_date: Option<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoolBalance {
    pub available_pool: i128,
    pub token_balance: i128,
}

#[contracttype]
enum DataKey {
    Community,
    Member(Address),
    Request(u64),
    Round(u64),
    Commitment(u64, Address),
    Reveal(u64, Address),
}

#[contractevent]
pub struct CommunityCreated {
    #[topic]
    pub admin: Address,
}

#[contractevent]
pub struct MemberJoined {
    #[topic]
    pub member: Address,
}

#[contractevent]
pub struct ContributionMade {
    #[topic]
    pub member: Address,
    pub amount: i128,
}

#[contractevent]
pub struct CapitalRequested {
    #[topic]
    pub member: Address,
    pub request_id: u64,
}

#[contractevent]
pub struct RoundCreated {
    pub round_id: u64,
}

fn bump_instance(env: &Env) {
    let max = env.storage().max_ttl();
    env.storage().instance().extend_ttl(max / 2, max);
}

fn bump_persistent(env: &Env, key: &DataKey) {
    let max = env.storage().max_ttl();
    env.storage().persistent().extend_ttl(key, max / 2, max);
}

fn load_community(env: &Env) -> Result<Community, Error> {
    env.storage()
        .instance()
        .get(&DataKey::Community)
        .ok_or(Error::NotInitialized)
}

fn save_community(env: &Env, community: &Community) {
    env.storage().instance().set(&DataKey::Community, community);
    bump_instance(env);
}

fn load_member(env: &Env, address: &Address) -> Option<Member> {
    env.storage()
        .persistent()
        .get(&DataKey::Member(address.clone()))
}

fn save_member(env: &Env, address: &Address, member: &Member) {
    let key = DataKey::Member(address.clone());
    env.storage().persistent().set(&key, member);
    bump_persistent(env, &key);
}

fn load_request(env: &Env, id: u64) -> Result<CapitalRequest, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::Request(id))
        .ok_or(Error::RequestNotFound)
}

fn save_request(env: &Env, request: &CapitalRequest) {
    let key = DataKey::Request(request.id);
    env.storage().persistent().set(&key, request);
    bump_persistent(env, &key);
}

fn load_round(env: &Env, id: u64) -> Result<Round, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::Round(id))
        .ok_or(Error::RoundNotFound)
}

fn save_round(env: &Env, round: &Round) {
    let key = DataKey::Round(round.id);
    env.storage().persistent().set(&key, round);
    bump_persistent(env, &key);
}

fn request_for_participant(
    env: &Env,
    round: &Round,
    participant: &Address,
) -> Result<CapitalRequest, Error> {
    for request_id in round.request_ids.iter() {
        let request = load_request(env, request_id)?;
        if request.member == *participant {
            return Ok(request);
        }
    }
    Err(Error::NotParticipant)
}

fn eligibility(community: &Community, member: Option<&Member>, amount: i128) -> EligibilityResult {
    let is_member = member.is_some();
    let active = member.is_some_and(|m| m.active);
    let contributions_completed = member.map_or(0, |m| m.contributions_completed);
    let contribution_requirement_met =
        is_member && contributions_completed >= community.minimum_contributions;
    let no_unresolved_default = member.is_some_and(|m| !m.unresolved_default);
    let no_outstanding_financing = member.is_some_and(|m| m.outstanding_financing == 0);
    let valid_amount = amount > 0;
    let within_financing_limit = amount <= community.financing_limit;
    EligibilityResult {
        eligible: is_member
            && active
            && contribution_requirement_met
            && no_unresolved_default
            && no_outstanding_financing
            && valid_amount
            && within_financing_limit,
        is_member,
        active,
        contribution_requirement_met,
        no_unresolved_default,
        no_outstanding_financing,
        valid_amount,
        within_financing_limit,
        contributions_completed,
        minimum_contributions: community.minimum_contributions,
    }
}

#[contract]
pub struct CommunityPool;

#[contractimpl]
impl CommunityPool {
    pub fn version() -> u32 {
        1
    }

    pub fn create_community(
        env: Env,
        admin: Address,
        asset: Address,
        contribution_amount: i128,
        minimum_contributions: u32,
        financing_limit: i128,
    ) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Community) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        if contribution_amount <= 0 || financing_limit <= 0 || minimum_contributions == 0 {
            return Err(Error::InvalidAmount);
        }
        if TokenClient::new(&env, &asset).try_decimals().is_err() {
            return Err(Error::InvalidAsset);
        }
        save_community(
            &env,
            &Community {
                initialized: true,
                admin: admin.clone(),
                asset,
                contribution_amount,
                minimum_contributions,
                financing_limit,
                member_count: 0,
                available_pool: 0,
                request_counter: 0,
                round_counter: 0,
            },
        );
        CommunityCreated { admin }.publish(&env);
        Ok(())
    }

    pub fn join(env: Env, member: Address) -> Result<(), Error> {
        let mut community = load_community(&env)?;
        member.require_auth();
        if load_member(&env, &member).is_some() {
            return Err(Error::MemberAlreadyExists);
        }
        community.member_count = community
            .member_count
            .checked_add(1)
            .ok_or(Error::Overflow)?;
        save_member(
            &env,
            &member,
            &Member {
                active: true,
                contributions_completed: 0,
                contributions_missed: 0,
                total_contributed: 0,
                financing_rounds_completed: 0,
                repayments_completed: 0,
                repayments_missed: 0,
                defaults: 0,
                unresolved_default: false,
                outstanding_financing: 0,
            },
        );
        save_community(&env, &community);
        MemberJoined { member }.publish(&env);
        Ok(())
    }

    pub fn contribute(env: Env, member: Address) -> Result<(), Error> {
        let mut community = load_community(&env)?;
        member.require_auth();
        let mut state = load_member(&env, &member).ok_or(Error::MemberNotFound)?;
        if !state.active {
            return Err(Error::InactiveMember);
        }
        let completed = state
            .contributions_completed
            .checked_add(1)
            .ok_or(Error::Overflow)?;
        let total = state
            .total_contributed
            .checked_add(community.contribution_amount)
            .ok_or(Error::Overflow)?;
        let available = community
            .available_pool
            .checked_add(community.contribution_amount)
            .ok_or(Error::Overflow)?;
        let destination = MuxedAddress::from(env.current_contract_address());
        TokenClient::new(&env, &community.asset).transfer(
            &member,
            &destination,
            &community.contribution_amount,
        );
        state.contributions_completed = completed;
        state.total_contributed = total;
        community.available_pool = available;
        save_member(&env, &member, &state);
        save_community(&env, &community);
        ContributionMade {
            member,
            amount: community.contribution_amount,
        }
        .publish(&env);
        Ok(())
    }

    pub fn request_capital(
        env: Env,
        member: Address,
        amount: i128,
        purpose_hash: BytesN<32>,
    ) -> Result<u64, Error> {
        let mut community = load_community(&env)?;
        member.require_auth();
        if amount <= 0 || amount > community.financing_limit {
            return Err(Error::InvalidAmount);
        }
        let state = load_member(&env, &member).ok_or(Error::MemberNotFound)?;
        if !state.active {
            return Err(Error::InactiveMember);
        }
        if !eligibility(&community, Some(&state), amount).eligible {
            return Err(Error::NotEligible);
        }
        let id = community
            .request_counter
            .checked_add(1)
            .ok_or(Error::Overflow)?;
        save_request(
            &env,
            &CapitalRequest {
                id,
                member: member.clone(),
                maximum_amount: amount,
                purpose_hash,
                status: RequestStatus::Pending,
            },
        );
        community.request_counter = id;
        save_community(&env, &community);
        CapitalRequested {
            member,
            request_id: id,
        }
        .publish(&env);
        Ok(id)
    }

    pub fn create_round(
        env: Env,
        admin: Address,
        request_ids: Vec<u64>,
        commit_deadline: u64,
        reveal_deadline: u64,
    ) -> Result<u64, Error> {
        let mut community = load_community(&env)?;
        if admin != community.admin {
            return Err(Error::Unauthorized);
        }
        admin.require_auth();
        if request_ids.is_empty() || request_ids.len() > MAX_ROUND_REQUESTS {
            return Err(Error::InvalidRoundSize);
        }
        if commit_deadline <= env.ledger().timestamp() || reveal_deadline <= commit_deadline {
            return Err(Error::InvalidDeadline);
        }
        let token_balance =
            TokenClient::new(&env, &community.asset).balance(&env.current_contract_address());
        let available_capital = core::cmp::min(community.available_pool, token_balance);
        if available_capital <= 0 {
            return Err(Error::InsufficientPoolBalance);
        }
        let mut participants = Vec::new(&env);
        for request_id in request_ids.iter() {
            let request = load_request(&env, request_id)?;
            if request.status != RequestStatus::Pending {
                return Err(Error::InvalidRequestStatus);
            }
            if participants.contains(&request.member) {
                return Err(Error::DuplicateParticipant);
            }
            let member = load_member(&env, &request.member);
            if !eligibility(&community, member.as_ref(), request.maximum_amount).eligible {
                return Err(Error::NotEligible);
            }
            participants.push_back(request.member);
        }
        let id = community
            .round_counter
            .checked_add(1)
            .ok_or(Error::Overflow)?;
        for request_id in request_ids.iter() {
            let mut request = load_request(&env, request_id)?;
            request.status = RequestStatus::IncludedInRound;
            save_request(&env, &request);
        }
        let round = Round {
            id,
            request_ids,
            participants,
            available_capital,
            commit_deadline,
            reveal_deadline,
            status: RoundStatus::Commit,
            winner: None,
            winning_bid: None,
            settled: false,
            valid_reveal_count: 0,
            outstanding_amount: 0,
            due_date: None,
        };
        save_round(&env, &round);
        community.round_counter = id;
        save_community(&env, &community);
        RoundCreated { round_id: id }.publish(&env);
        Ok(id)
    }

    pub fn commit_bid(
        env: Env,
        round_id: u64,
        participant: Address,
        commitment: BytesN<32>,
    ) -> Result<(), Error> {
        let community = load_community(&env)?;
        participant.require_auth();
        let round = load_round(&env, round_id)?;
        if round.status != RoundStatus::Commit {
            return Err(Error::InvalidRoundStatus);
        }
        if env.ledger().timestamp() >= round.commit_deadline {
            return Err(Error::CommitClosed);
        }
        if !round.participants.contains(&participant) {
            return Err(Error::NotParticipant);
        }
        let request = request_for_participant(&env, &round, &participant)?;
        let member = load_member(&env, &participant);
        if !eligibility(&community, member.as_ref(), request.maximum_amount).eligible {
            return Err(Error::NotEligible);
        }
        let key = DataKey::Commitment(round_id, participant);
        if env.storage().persistent().has(&key) {
            return Err(Error::CommitmentAlreadyExists);
        }
        env.storage().persistent().set(&key, &commitment);
        bump_persistent(&env, &key);
        Ok(())
    }

    /// Permissionless phase transition after the commit deadline.
    pub fn start_reveal(env: Env, round_id: u64) -> Result<(), Error> {
        load_community(&env)?;
        let mut round = load_round(&env, round_id)?;
        if round.status != RoundStatus::Commit {
            return Err(Error::InvalidRoundStatus);
        }
        if env.ledger().timestamp() < round.commit_deadline {
            return Err(Error::CommitClosed);
        }
        round.status = RoundStatus::Reveal;
        save_round(&env, &round);
        Ok(())
    }

    pub fn reveal_bid(
        env: Env,
        round_id: u64,
        participant: Address,
        bid_amount: i128,
        secret: BytesN<32>,
    ) -> Result<(), Error> {
        load_community(&env)?;
        participant.require_auth();
        let mut round = load_round(&env, round_id)?;
        if round.status != RoundStatus::Reveal {
            return Err(Error::RevealNotOpen);
        }
        let now = env.ledger().timestamp();
        if now < round.commit_deadline {
            return Err(Error::RevealNotOpen);
        }
        if now >= round.reveal_deadline {
            return Err(Error::RevealClosed);
        }
        if !round.participants.contains(&participant) {
            return Err(Error::NotParticipant);
        }
        let reveal_key = DataKey::Reveal(round_id, participant.clone());
        if env.storage().persistent().has(&reveal_key) {
            return Err(Error::RevealAlreadyExists);
        }
        let commit_key = DataKey::Commitment(round_id, participant.clone());
        let stored: BytesN<32> = env
            .storage()
            .persistent()
            .get(&commit_key)
            .ok_or(Error::CommitmentNotFound)?;
        let request = request_for_participant(&env, &round, &participant)?;
        if bid_amount <= 0 || bid_amount > request.maximum_amount {
            return Err(Error::InvalidBid);
        }
        if bid_amount > round.available_capital {
            return Err(Error::InsufficientPoolBalance);
        }
        if commitment::hash(&env, round_id, &participant, bid_amount, &secret) != stored {
            return Err(Error::CommitmentMismatch);
        }
        let next_count = round
            .valid_reveal_count
            .checked_add(1)
            .ok_or(Error::Overflow)?;
        env.storage().persistent().set(&reveal_key, &bid_amount);
        bump_persistent(&env, &reveal_key);
        round.valid_reveal_count = next_count;
        save_round(&env, &round);
        Ok(())
    }

    pub fn finalize_round(env: Env, round_id: u64) -> Result<(), Error> {
        load_community(&env)?;
        let mut round = load_round(&env, round_id)?;
        if round.status != RoundStatus::Reveal {
            return Err(Error::InvalidRoundStatus);
        }
        if env.ledger().timestamp() < round.reveal_deadline {
            return Err(Error::FinalizationTooEarly);
        }
        let mut best_wallet: Option<Address> = None;
        let mut best_amount: Option<i128> = None;
        for participant in round.participants.iter() {
            let bid: Option<i128> = env
                .storage()
                .persistent()
                .get(&DataKey::Reveal(round_id, participant.clone()));
            if let Some(amount) = bid {
                let wins = match (&best_wallet, best_amount) {
                    (Some(current_wallet), Some(current_amount)) => {
                        amount < current_amount
                            || (amount == current_amount
                                && participant.to_string() < current_wallet.to_string())
                    }
                    _ => true,
                };
                if wins {
                    best_wallet = Some(participant);
                    best_amount = Some(amount);
                }
            }
        }
        round.winner = best_wallet;
        round.winning_bid = best_amount;
        round.status = RoundStatus::Finalized;
        save_round(&env, &round);
        Ok(())
    }

    pub fn settle(env: Env, round_id: u64, winner: Address, due_date: u64) -> Result<(), Error> {
        let mut community = load_community(&env)?;
        winner.require_auth();
        let mut round = load_round(&env, round_id)?;
        if round.status == RoundStatus::Settled || round.settled {
            return Err(Error::AlreadySettled);
        }
        if round.status != RoundStatus::Finalized {
            return Err(Error::InvalidRoundStatus);
        }
        if round.winner.as_ref() != Some(&winner) {
            return Err(Error::NoWinner);
        }
        if due_date <= env.ledger().timestamp() {
            return Err(Error::InvalidDeadline);
        }
        let amount = round.winning_bid.ok_or(Error::NoWinner)?;
        if amount <= 0 || amount > community.available_pool {
            return Err(Error::InsufficientPoolBalance);
        }
        let token = TokenClient::new(&env, &community.asset);
        if token.balance(&env.current_contract_address()) < amount {
            return Err(Error::InsufficientPoolBalance);
        }
        let mut member = load_member(&env, &winner).ok_or(Error::MemberNotFound)?;
        if !member.active || member.unresolved_default || member.outstanding_financing != 0 {
            return Err(Error::NotEligible);
        }
        let mut request = request_for_participant(&env, &round, &winner)?;
        if request.status != RequestStatus::IncludedInRound {
            return Err(Error::InvalidRequestStatus);
        }
        let available = community
            .available_pool
            .checked_sub(amount)
            .ok_or(Error::Overflow)?;
        let destination = MuxedAddress::from(&winner);
        token.transfer(&env.current_contract_address(), &destination, &amount);
        community.available_pool = available;
        member.outstanding_financing = amount;
        request.status = RequestStatus::Funded;
        round.outstanding_amount = amount;
        round.due_date = Some(due_date);
        round.settled = true;
        round.status = RoundStatus::Settled;
        save_member(&env, &winner, &member);
        save_request(&env, &request);
        save_round(&env, &round);
        save_community(&env, &community);
        Ok(())
    }

    pub fn repay(
        env: Env,
        round_id: u64,
        member_address: Address,
        amount: i128,
    ) -> Result<(), Error> {
        let mut community = load_community(&env)?;
        member_address.require_auth();
        let mut round = load_round(&env, round_id)?;
        if round.status != RoundStatus::Settled || round.winner.as_ref() != Some(&member_address) {
            return Err(Error::InvalidRoundStatus);
        }
        if amount <= 0 || amount > round.outstanding_amount {
            return Err(Error::InvalidRepayment);
        }
        let mut member = load_member(&env, &member_address).ok_or(Error::MemberNotFound)?;
        if amount > member.outstanding_financing {
            return Err(Error::InvalidRepayment);
        }
        let available = community
            .available_pool
            .checked_add(amount)
            .ok_or(Error::Overflow)?;
        let outstanding = round
            .outstanding_amount
            .checked_sub(amount)
            .ok_or(Error::Overflow)?;
        let member_outstanding = member
            .outstanding_financing
            .checked_sub(amount)
            .ok_or(Error::Overflow)?;
        let (completed_rounds, completed_repayments) = if outstanding == 0 {
            (
                member
                    .financing_rounds_completed
                    .checked_add(1)
                    .ok_or(Error::Overflow)?,
                member
                    .repayments_completed
                    .checked_add(1)
                    .ok_or(Error::Overflow)?,
            )
        } else {
            (
                member.financing_rounds_completed,
                member.repayments_completed,
            )
        };
        let destination = MuxedAddress::from(env.current_contract_address());
        TokenClient::new(&env, &community.asset).transfer(&member_address, &destination, &amount);
        community.available_pool = available;
        round.outstanding_amount = outstanding;
        member.outstanding_financing = member_outstanding;
        member.financing_rounds_completed = completed_rounds;
        member.repayments_completed = completed_repayments;
        if outstanding == 0 {
            let mut request = request_for_participant(&env, &round, &member_address)?;
            if request.status != RequestStatus::Funded {
                return Err(Error::InvalidRequestStatus);
            }
            request.status = RequestStatus::Repaid;
            save_request(&env, &request);
        }
        save_member(&env, &member_address, &member);
        save_round(&env, &round);
        save_community(&env, &community);
        Ok(())
    }

    pub fn get_commitment(
        env: Env,
        round_id: u64,
        participant: Address,
    ) -> Result<Option<BytesN<32>>, Error> {
        load_community(&env)?;
        load_round(&env, round_id)?;
        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::Commitment(round_id, participant)))
    }

    pub fn get_reveal(
        env: Env,
        round_id: u64,
        participant: Address,
    ) -> Result<Option<i128>, Error> {
        load_community(&env)?;
        load_round(&env, round_id)?;
        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::Reveal(round_id, participant)))
    }

    pub fn get_community(env: Env) -> Result<Community, Error> {
        load_community(&env)
    }

    pub fn get_pool_balance(env: Env) -> Result<PoolBalance, Error> {
        let community = load_community(&env)?;
        Ok(PoolBalance {
            available_pool: community.available_pool,
            token_balance: TokenClient::new(&env, &community.asset)
                .balance(&env.current_contract_address()),
        })
    }

    pub fn get_member(env: Env, address: Address) -> Result<Member, Error> {
        load_community(&env)?;
        load_member(&env, &address).ok_or(Error::MemberNotFound)
    }

    pub fn is_member(env: Env, address: Address) -> Result<bool, Error> {
        load_community(&env)?;
        Ok(load_member(&env, &address).is_some())
    }

    pub fn get_eligibility(
        env: Env,
        address: Address,
        amount: i128,
    ) -> Result<EligibilityResult, Error> {
        let community = load_community(&env)?;
        let member = load_member(&env, &address);
        Ok(eligibility(&community, member.as_ref(), amount))
    }

    pub fn get_request(env: Env, request_id: u64) -> Result<CapitalRequest, Error> {
        load_community(&env)?;
        load_request(&env, request_id)
    }

    pub fn get_round(env: Env, round_id: u64) -> Result<Round, Error> {
        load_community(&env)?;
        load_round(&env, round_id)
    }
}

#[cfg(test)]
mod tests;
