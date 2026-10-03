#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, token::TokenClient,
    Address, BytesN, Env, MuxedAddress, Vec,
};

// Hash inputs stay private until reveal; the preimage is never stored.
mod commitment;

const MAX_ROUND_REQUESTS: u32 = 10;
const MAX_CYCLE_MEMBERS: u32 = 10;

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
    CycleNotFound = 31,
    CycleActive = 32,
    CycleComplete = 33,
    NotCycleMember = 34,
    ContributionAlreadyMade = 35,
    ContributionsIncomplete = 36,
    RoundAlreadyActive = 37,
    AlreadyPaidOut = 38,
    NoDiscountToClaim = 39,
    InvalidCycleSize = 40,
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
    pub cycle_counter: u64,
    pub current_cycle_id: Option<u64>,
    pub reserved_pool: i128,
    pub discount_liability: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Member {
    pub active: bool,
    pub contributions_completed: u32,
    pub contributions_missed: u32,
    pub total_contributed: i128,
    pub cycles_joined: u32,
    pub cycles_completed: u32,
    pub cycle_ids: Vec<u64>,
    pub payouts_received: u32,
    pub total_payouts_received: i128,
    pub post_payout_contributions: u32,
    pub discounts_earned: i128,
    pub discounts_claimed: i128,
    pub defaults: u32,
    pub unresolved_default: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cycle {
    pub id: u64,
    pub members: Vec<Address>,
    pub current_round_number: u32,
    pub active_round_id: Option<u64>,
    pub payouts_completed: u32,
    pub complete: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CycleMember {
    pub cycle_id: u64,
    pub member: Address,
    pub payout_received: bool,
    pub expected_contributions: u32,
    pub completed_contributions: u32,
    pub contributions_before_payout: u32,
    pub contributions_after_payout: u32,
    pub obligation_complete: bool,
    pub payout_amount: i128,
    pub payout_round_number: Option<u32>,
    pub claimable_discount: i128,
    pub discounts_claimed: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinancialHistory {
    pub member: Address,
    pub contributions_completed: u32,
    pub total_contributed: i128,
    pub cycles_joined: u32,
    pub cycles_completed: u32,
    pub cycle_ids: Vec<u64>,
    pub payouts_received: u32,
    pub total_payouts_received: i128,
    pub post_payout_contributions: u32,
    pub discounts_earned: i128,
    pub discounts_claimed: i128,
    pub current_cycle_id: Option<u64>,
    pub current_cycle_payout_received: bool,
    pub current_cycle_expected: u32,
    pub current_cycle_completed: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CycleFinancialHistory {
    pub cycle_id: u64,
    pub member: Address,
    pub cycle_complete: bool,
    pub expected_contributions: u32,
    pub completed_contributions: u32,
    pub contributions_before_payout: u32,
    pub contributions_after_payout: u32,
    pub post_payout_expected_to_date: u32,
    pub post_payout_required_total: u32,
    pub payout_received: bool,
    pub payout_amount: i128,
    pub payout_round_number: Option<u32>,
    pub discounts_earned: i128,
    pub discounts_claimed: i128,
    pub claimable_discount: i128,
    pub obligation_complete: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EligibilityResult {
    pub eligible: bool,
    pub is_member: bool,
    pub active: bool,
    pub contribution_requirement_met: bool,
    pub no_unresolved_default: bool,
    pub in_cycle: bool,
    pub current_round_contribution_met: bool,
    pub payout_not_received: bool,
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
    PaidOut,
    NotSelected,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapitalRequest {
    pub id: u64,
    pub member: Address,
    pub maximum_amount: i128,
    pub purpose_hash: BytesN<32>,
    pub status: RequestStatus,
    pub cycle_id: u64,
    pub cycle_round_number: u32,
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
    pub cycle_id: u64,
    pub cycle_round_number: u32,
    pub request_ids: Vec<u64>,
    pub participants: Vec<Address>,
    pub available_capital: i128,
    pub pot: i128,
    pub commit_deadline: u64,
    pub reveal_deadline: u64,
    pub status: RoundStatus,
    pub winner: Option<Address>,
    pub winning_bid: Option<i128>,
    pub settled: bool,
    pub valid_reveal_count: u32,
    pub auction_discount: i128,
    pub discount_per_member: i128,
    pub discount_remainder: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoolBalance {
    pub available_pool: i128,
    pub reserved_pool: i128,
    pub discount_liability: i128,
    pub token_balance: i128,
}

#[contracttype]
enum DataKey {
    Community,
    Member(Address),
    Request(u64),
    Round(u64),
    Cycle(u64),
    CycleMember(u64, Address),
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

fn load_cycle(env: &Env, id: u64) -> Result<Cycle, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::Cycle(id))
        .ok_or(Error::CycleNotFound)
}

fn save_cycle(env: &Env, cycle: &Cycle) {
    let key = DataKey::Cycle(cycle.id);
    env.storage().persistent().set(&key, cycle);
    bump_persistent(env, &key);
}

fn load_cycle_member(env: &Env, cycle_id: u64, address: &Address) -> Option<CycleMember> {
    env.storage()
        .persistent()
        .get(&DataKey::CycleMember(cycle_id, address.clone()))
}

fn save_cycle_member(env: &Env, state: &CycleMember) {
    let key = DataKey::CycleMember(state.cycle_id, state.member.clone());
    env.storage().persistent().set(&key, state);
    bump_persistent(env, &key);
}

fn current_cycle(env: &Env, community: &Community) -> Result<Cycle, Error> {
    load_cycle(env, community.current_cycle_id.ok_or(Error::CycleNotFound)?)
}

fn accounted_total(community: &Community) -> Result<i128, Error> {
    community
        .available_pool
        .checked_add(community.reserved_pool)
        .and_then(|value| value.checked_add(community.discount_liability))
        .ok_or(Error::Overflow)
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

fn eligibility(
    community: &Community,
    member: Option<&Member>,
    cycle: Option<&Cycle>,
    cycle_member: Option<&CycleMember>,
    amount: i128,
) -> EligibilityResult {
    let is_member = member.is_some();
    let active = member.is_some_and(|m| m.active);
    let contributions_completed = member.map_or(0, |m| m.contributions_completed);
    let required = cycle.map_or(community.minimum_contributions, |c| {
        core::cmp::min(community.minimum_contributions, c.current_round_number)
    });
    let contribution_requirement_met = is_member && contributions_completed >= required;
    let no_unresolved_default = member.is_some_and(|m| !m.unresolved_default);
    let in_cycle = cycle_member.is_some();
    let current_round_contribution_met = cycle_member.is_some_and(|m| {
        cycle.is_some_and(|c| m.completed_contributions == c.current_round_number)
    });
    let payout_not_received = cycle_member.is_some_and(|m| !m.payout_received);
    let valid_amount = amount > 0;
    let within_financing_limit = amount <= community.financing_limit;
    EligibilityResult {
        eligible: is_member
            && active
            && contribution_requirement_met
            && no_unresolved_default
            && in_cycle
            && current_round_contribution_met
            && payout_not_received
            && cycle.is_some_and(|c| !c.complete)
            && valid_amount
            && within_financing_limit,
        is_member,
        active,
        contribution_requirement_met,
        no_unresolved_default,
        in_cycle,
        current_round_contribution_met,
        payout_not_received,
        valid_amount,
        within_financing_limit,
        contributions_completed,
        minimum_contributions: required,
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
                cycle_counter: 0,
                current_cycle_id: None,
                reserved_pool: 0,
                discount_liability: 0,
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
                cycles_joined: 0,
                cycles_completed: 0,
                cycle_ids: Vec::new(&env),
                payouts_received: 0,
                total_payouts_received: 0,
                post_payout_contributions: 0,
                discounts_earned: 0,
                discounts_claimed: 0,
                defaults: 0,
                unresolved_default: false,
            },
        );
        save_community(&env, &community);
        MemberJoined { member }.publish(&env);
        Ok(())
    }

    pub fn create_cycle(env: Env, admin: Address, members: Vec<Address>) -> Result<u64, Error> {
        let mut community = load_community(&env)?;
        if admin != community.admin {
            return Err(Error::Unauthorized);
        }
        admin.require_auth();
        if members.is_empty() || members.len() > MAX_CYCLE_MEMBERS {
            return Err(Error::InvalidCycleSize);
        }
        if let Some(id) = community.current_cycle_id {
            if !load_cycle(&env, id)?.complete {
                return Err(Error::CycleActive);
            }
        }
        for (index, address) in members.iter().enumerate() {
            if members.first_index_of(&address) != Some(index as u32) {
                return Err(Error::DuplicateParticipant);
            }
            if !load_member(&env, &address).is_some_and(|member| member.active) {
                return Err(Error::NotEligible);
            }
        }
        let id = community
            .cycle_counter
            .checked_add(1)
            .ok_or(Error::Overflow)?;
        for address in members.iter() {
            let mut history = load_member(&env, &address).ok_or(Error::MemberNotFound)?;
            history.cycles_joined = history
                .cycles_joined
                .checked_add(1)
                .ok_or(Error::Overflow)?;
            history.cycle_ids.push_back(id);
            save_member(&env, &address, &history);
            save_cycle_member(
                &env,
                &CycleMember {
                    cycle_id: id,
                    member: address,
                    payout_received: false,
                    expected_contributions: 1,
                    completed_contributions: 0,
                    contributions_before_payout: 0,
                    contributions_after_payout: 0,
                    obligation_complete: false,
                    payout_amount: 0,
                    payout_round_number: None,
                    claimable_discount: 0,
                    discounts_claimed: 0,
                },
            );
        }
        save_cycle(
            &env,
            &Cycle {
                id,
                members,
                current_round_number: 1,
                active_round_id: None,
                payouts_completed: 0,
                complete: false,
            },
        );
        community.cycle_counter = id;
        community.current_cycle_id = Some(id);
        save_community(&env, &community);
        Ok(id)
    }

    pub fn contribute(env: Env, member: Address) -> Result<(), Error> {
        let mut community = load_community(&env)?;
        member.require_auth();
        let mut state = load_member(&env, &member).ok_or(Error::MemberNotFound)?;
        if !state.active {
            return Err(Error::InactiveMember);
        }
        let cycle = current_cycle(&env, &community)?;
        if cycle.complete {
            return Err(Error::CycleComplete);
        }
        if cycle.active_round_id.is_some() {
            return Err(Error::RoundAlreadyActive);
        }
        let mut cycle_state =
            load_cycle_member(&env, cycle.id, &member).ok_or(Error::NotCycleMember)?;
        if cycle_state.completed_contributions >= cycle.current_round_number {
            return Err(Error::ContributionAlreadyMade);
        }
        if cycle_state.completed_contributions + 1 != cycle.current_round_number {
            return Err(Error::ContributionsIncomplete);
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
        cycle_state.completed_contributions = cycle.current_round_number;
        if cycle_state.payout_received {
            cycle_state.contributions_after_payout = cycle_state
                .contributions_after_payout
                .checked_add(1)
                .ok_or(Error::Overflow)?;
            state.post_payout_contributions = state
                .post_payout_contributions
                .checked_add(1)
                .ok_or(Error::Overflow)?;
        } else {
            cycle_state.contributions_before_payout = cycle_state
                .contributions_before_payout
                .checked_add(1)
                .ok_or(Error::Overflow)?;
        }
        community.available_pool = available;
        save_member(&env, &member, &state);
        save_cycle_member(&env, &cycle_state);
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
        let cycle = current_cycle(&env, &community)?;
        if cycle.active_round_id.is_some() {
            return Err(Error::RoundAlreadyActive);
        }
        let cycle_member = load_cycle_member(&env, cycle.id, &member);
        if !eligibility(
            &community,
            Some(&state),
            Some(&cycle),
            cycle_member.as_ref(),
            amount,
        )
        .eligible
        {
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
                cycle_id: cycle.id,
                cycle_round_number: cycle.current_round_number,
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
        let mut cycle = current_cycle(&env, &community)?;
        if cycle.complete {
            return Err(Error::CycleComplete);
        }
        if cycle.active_round_id.is_some() {
            return Err(Error::RoundAlreadyActive);
        }
        for address in cycle.members.iter() {
            let state = load_cycle_member(&env, cycle.id, &address).ok_or(Error::NotCycleMember)?;
            if state.completed_contributions != cycle.current_round_number {
                return Err(Error::ContributionsIncomplete);
            }
        }
        let pot = community
            .contribution_amount
            .checked_mul(i128::from(cycle.members.len()))
            .ok_or(Error::Overflow)?;
        let token_balance =
            TokenClient::new(&env, &community.asset).balance(&env.current_contract_address());
        if community.available_pool < pot || token_balance < accounted_total(&community)? {
            return Err(Error::InsufficientPoolBalance);
        }
        let mut participants = Vec::new(&env);
        for request_id in request_ids.iter() {
            let request = load_request(&env, request_id)?;
            if request.cycle_id != cycle.id
                || request.cycle_round_number != cycle.current_round_number
            {
                return Err(Error::InvalidRequestStatus);
            }
            if request.status != RequestStatus::Pending {
                return Err(Error::InvalidRequestStatus);
            }
            if participants.contains(&request.member) {
                return Err(Error::DuplicateParticipant);
            }
            let member = load_member(&env, &request.member);
            let cycle_member = load_cycle_member(&env, cycle.id, &request.member);
            if !eligibility(
                &community,
                member.as_ref(),
                Some(&cycle),
                cycle_member.as_ref(),
                request.maximum_amount,
            )
            .eligible
            {
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
            cycle_id: cycle.id,
            cycle_round_number: cycle.current_round_number,
            request_ids,
            participants,
            available_capital: pot,
            pot,
            commit_deadline,
            reveal_deadline,
            status: RoundStatus::Commit,
            winner: None,
            winning_bid: None,
            settled: false,
            valid_reveal_count: 0,
            auction_discount: 0,
            discount_per_member: 0,
            discount_remainder: 0,
        };
        save_round(&env, &round);
        cycle.active_round_id = Some(id);
        save_cycle(&env, &cycle);
        community.available_pool = community
            .available_pool
            .checked_sub(pot)
            .ok_or(Error::Overflow)?;
        community.reserved_pool = community
            .reserved_pool
            .checked_add(pot)
            .ok_or(Error::Overflow)?;
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
        let cycle = load_cycle(&env, round.cycle_id)?;
        let cycle_member = load_cycle_member(&env, cycle.id, &participant);
        if !eligibility(
            &community,
            member.as_ref(),
            Some(&cycle),
            cycle_member.as_ref(),
            request.maximum_amount,
        )
        .eligible
        {
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
        if round.winner.is_none() {
            let mut community = load_community(&env)?;
            let mut cycle = load_cycle(&env, round.cycle_id)?;
            community.reserved_pool = community
                .reserved_pool
                .checked_sub(round.pot)
                .ok_or(Error::Overflow)?;
            community.available_pool = community
                .available_pool
                .checked_add(round.pot)
                .ok_or(Error::Overflow)?;
            cycle.active_round_id = None;
            for request_id in round.request_ids.iter() {
                let mut request = load_request(&env, request_id)?;
                request.status = RequestStatus::Pending;
                save_request(&env, &request);
            }
            save_cycle(&env, &cycle);
            save_community(&env, &community);
        }
        save_round(&env, &round);
        Ok(())
    }

    pub fn settle(env: Env, round_id: u64, winner: Address) -> Result<(), Error> {
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
        let amount = round.winning_bid.ok_or(Error::NoWinner)?;
        let mut cycle = load_cycle(&env, round.cycle_id)?;
        if cycle.complete || cycle.active_round_id != Some(round_id) {
            return Err(Error::InvalidRoundStatus);
        }
        let winner_state =
            load_cycle_member(&env, cycle.id, &winner).ok_or(Error::NotCycleMember)?;
        if winner_state.payout_received {
            return Err(Error::AlreadyPaidOut);
        }
        let member = load_member(&env, &winner).ok_or(Error::MemberNotFound)?;
        if !member.active || member.unresolved_default {
            return Err(Error::NotEligible);
        }
        if amount <= 0 || amount > round.pot || community.reserved_pool < round.pot {
            return Err(Error::InsufficientPoolBalance);
        }
        let token = TokenClient::new(&env, &community.asset);
        if token.balance(&env.current_contract_address()) < accounted_total(&community)? {
            return Err(Error::InsufficientPoolBalance);
        }
        let discount = round.pot.checked_sub(amount).ok_or(Error::Overflow)?;
        let member_count = i128::from(cycle.members.len());
        let equal_share = discount / member_count;
        let remainder = u32::try_from(discount % member_count).map_err(|_| Error::Overflow)?;
        let credited = equal_share
            .checked_mul(member_count)
            .and_then(|value| value.checked_add(i128::from(remainder)))
            .ok_or(Error::Overflow)?;
        if amount.checked_add(credited) != Some(round.pot) {
            return Err(Error::Overflow);
        }
        let next_payouts = cycle
            .payouts_completed
            .checked_add(1)
            .ok_or(Error::Overflow)?;
        let cycle_complete = next_payouts == cycle.members.len();
        for address in cycle.members.iter() {
            let mut state =
                load_cycle_member(&env, cycle.id, &address).ok_or(Error::NotCycleMember)?;
            let mut history = load_member(&env, &address).ok_or(Error::MemberNotFound)?;
            let mut rank = 0u32;
            for other in cycle.members.iter() {
                if other.to_string() < address.to_string() {
                    rank = rank.checked_add(1).ok_or(Error::Overflow)?;
                }
            }
            let extra = if rank < remainder { 1 } else { 0 };
            let credit = equal_share + extra;
            state.claimable_discount = state
                .claimable_discount
                .checked_add(credit)
                .ok_or(Error::Overflow)?;
            history.discounts_earned = history
                .discounts_earned
                .checked_add(credit)
                .ok_or(Error::Overflow)?;
            if address == winner {
                state.payout_received = true;
                state.payout_amount = amount;
                state.payout_round_number = Some(cycle.current_round_number);
                history.payouts_received = history
                    .payouts_received
                    .checked_add(1)
                    .ok_or(Error::Overflow)?;
                history.total_payouts_received = history
                    .total_payouts_received
                    .checked_add(amount)
                    .ok_or(Error::Overflow)?;
            }
            if cycle_complete {
                state.obligation_complete =
                    state.completed_contributions == state.expected_contributions;
                if state.obligation_complete {
                    history.cycles_completed = history
                        .cycles_completed
                        .checked_add(1)
                        .ok_or(Error::Overflow)?;
                }
            } else {
                state.expected_contributions = state
                    .expected_contributions
                    .checked_add(1)
                    .ok_or(Error::Overflow)?;
            }
            save_cycle_member(&env, &state);
            save_member(&env, &address, &history);
        }
        community.reserved_pool = community
            .reserved_pool
            .checked_sub(round.pot)
            .ok_or(Error::Overflow)?;
        community.discount_liability = community
            .discount_liability
            .checked_add(discount)
            .ok_or(Error::Overflow)?;
        for request_id in round.request_ids.iter() {
            let mut request = load_request(&env, request_id)?;
            request.status = if request.member == winner {
                RequestStatus::PaidOut
            } else {
                RequestStatus::NotSelected
            };
            save_request(&env, &request);
        }
        cycle.payouts_completed = next_payouts;
        cycle.active_round_id = None;
        cycle.complete = cycle_complete;
        if !cycle_complete {
            cycle.current_round_number = cycle
                .current_round_number
                .checked_add(1)
                .ok_or(Error::Overflow)?;
        }
        round.auction_discount = discount;
        round.discount_per_member = equal_share;
        round.discount_remainder = remainder;
        round.settled = true;
        round.status = RoundStatus::Settled;
        token.transfer(
            &env.current_contract_address(),
            &MuxedAddress::from(&winner),
            &amount,
        );
        save_cycle(&env, &cycle);
        save_round(&env, &round);
        save_community(&env, &community);
        Ok(())
    }

    pub fn claim_discount(env: Env, cycle_id: u64, member: Address) -> Result<i128, Error> {
        let mut community = load_community(&env)?;
        member.require_auth();
        load_cycle(&env, cycle_id)?;
        let mut state = load_cycle_member(&env, cycle_id, &member).ok_or(Error::NotCycleMember)?;
        let amount = state.claimable_discount;
        if amount <= 0 {
            return Err(Error::NoDiscountToClaim);
        }
        let mut history = load_member(&env, &member).ok_or(Error::MemberNotFound)?;
        let token = TokenClient::new(&env, &community.asset);
        if token.balance(&env.current_contract_address()) < accounted_total(&community)? {
            return Err(Error::InsufficientPoolBalance);
        }
        let next_claimed = history
            .discounts_claimed
            .checked_add(amount)
            .ok_or(Error::Overflow)?;
        let cycle_claimed = state
            .discounts_claimed
            .checked_add(amount)
            .ok_or(Error::Overflow)?;
        let next_liability = community
            .discount_liability
            .checked_sub(amount)
            .ok_or(Error::Overflow)?;
        token.transfer(
            &env.current_contract_address(),
            &MuxedAddress::from(&member),
            &amount,
        );
        state.claimable_discount = 0;
        state.discounts_claimed = cycle_claimed;
        history.discounts_claimed = next_claimed;
        community.discount_liability = next_liability;
        save_cycle_member(&env, &state);
        save_member(&env, &member, &history);
        save_community(&env, &community);
        Ok(amount)
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
            reserved_pool: community.reserved_pool,
            discount_liability: community.discount_liability,
            token_balance: TokenClient::new(&env, &community.asset)
                .balance(&env.current_contract_address()),
        })
    }

    pub fn get_member(env: Env, address: Address) -> Result<Member, Error> {
        load_community(&env)?;
        load_member(&env, &address).ok_or(Error::MemberNotFound)
    }

    pub fn get_financial_history(env: Env, address: Address) -> Result<FinancialHistory, Error> {
        let community = load_community(&env)?;
        let member = load_member(&env, &address).ok_or(Error::MemberNotFound)?;
        let cycle_state = community
            .current_cycle_id
            .and_then(|id| load_cycle_member(&env, id, &address));
        Ok(FinancialHistory {
            member: address,
            contributions_completed: member.contributions_completed,
            total_contributed: member.total_contributed,
            cycles_joined: member.cycles_joined,
            cycles_completed: member.cycles_completed,
            cycle_ids: member.cycle_ids,
            payouts_received: member.payouts_received,
            total_payouts_received: member.total_payouts_received,
            post_payout_contributions: member.post_payout_contributions,
            discounts_earned: member.discounts_earned,
            discounts_claimed: member.discounts_claimed,
            current_cycle_id: cycle_state.as_ref().map(|state| state.cycle_id),
            current_cycle_payout_received: cycle_state
                .as_ref()
                .is_some_and(|state| state.payout_received),
            current_cycle_expected: cycle_state
                .as_ref()
                .map_or(0, |state| state.expected_contributions),
            current_cycle_completed: cycle_state
                .as_ref()
                .map_or(0, |state| state.completed_contributions),
        })
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
        let cycle = community
            .current_cycle_id
            .and_then(|id| load_cycle(&env, id).ok());
        let cycle_member = cycle
            .as_ref()
            .and_then(|state| load_cycle_member(&env, state.id, &address));
        Ok(eligibility(
            &community,
            member.as_ref(),
            cycle.as_ref(),
            cycle_member.as_ref(),
            amount,
        ))
    }

    pub fn get_cycle(env: Env, cycle_id: u64) -> Result<Cycle, Error> {
        load_community(&env)?;
        load_cycle(&env, cycle_id)
    }

    pub fn get_cycle_member(
        env: Env,
        cycle_id: u64,
        member: Address,
    ) -> Result<CycleMember, Error> {
        load_community(&env)?;
        load_cycle(&env, cycle_id)?;
        load_cycle_member(&env, cycle_id, &member).ok_or(Error::NotCycleMember)
    }

    pub fn get_cycle_history(
        env: Env,
        cycle_id: u64,
        member: Address,
    ) -> Result<CycleFinancialHistory, Error> {
        load_community(&env)?;
        let cycle = load_cycle(&env, cycle_id)?;
        let state = load_cycle_member(&env, cycle_id, &member).ok_or(Error::NotCycleMember)?;
        let post_payout_expected_to_date = state.payout_round_number.map_or(0, |round| {
            state.expected_contributions.saturating_sub(round)
        });
        let post_payout_required_total = state
            .payout_round_number
            .map_or(0, |round| cycle.members.len().saturating_sub(round));
        let discounts_earned = state
            .claimable_discount
            .checked_add(state.discounts_claimed)
            .ok_or(Error::Overflow)?;
        Ok(CycleFinancialHistory {
            cycle_id,
            member,
            cycle_complete: cycle.complete,
            expected_contributions: state.expected_contributions,
            completed_contributions: state.completed_contributions,
            contributions_before_payout: state.contributions_before_payout,
            contributions_after_payout: state.contributions_after_payout,
            post_payout_expected_to_date,
            post_payout_required_total,
            payout_received: state.payout_received,
            payout_amount: state.payout_amount,
            payout_round_number: state.payout_round_number,
            discounts_earned,
            discounts_claimed: state.discounts_claimed,
            claimable_discount: state.claimable_discount,
            obligation_complete: state.obligation_complete,
        })
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

#[cfg(test)]
mod auction_tests;
