extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, token::StellarAssetClient, vec};

const UNIT: i128 = 10_000_000;
const CONTRIBUTION: i128 = 5_000 * UNIT;
const POT: i128 = 50_000 * UNIT;

struct Fixture {
    env: Env,
    contract: Address,
    admin: Address,
    asset: Address,
    members: Vec<Address>,
}

impl Fixture {
    fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let asset = env
            .register_stellar_asset_contract_v2(admin.clone())
            .address();
        let contract = env.register(CommunityPool, ());
        let client = CommunityPoolClient::new(&env, &contract);
        client.create_community(&admin, &asset, &CONTRIBUTION, &3, &POT);
        let mut members = Vec::new(&env);
        let issuer = StellarAssetClient::new(&env, &asset);
        for _ in 0..10 {
            let address = Address::generate(&env);
            issuer.mint(&address, &POT);
            client.join(&address);
            members.push_back(address);
        }
        Self {
            env,
            contract,
            admin,
            asset,
            members,
        }
    }

    fn client(&self) -> CommunityPoolClient<'_> {
        CommunityPoolClient::new(&self.env, &self.contract)
    }

    fn first(&self) -> Address {
        self.members.get(0).unwrap()
    }

    fn start_cycle(&self) -> u64 {
        self.client().create_cycle(&self.admin, &self.members)
    }

    fn fund_round(&self) {
        for member in self.members.iter() {
            self.client().contribute(&member);
        }
    }

    fn purpose(&self) -> BytesN<32> {
        BytesN::from_array(&self.env, &[7; 32])
    }
}

#[test]
fn community_and_membership_initialize_once() {
    let f = Fixture::new();
    let state = f.client().get_community();
    assert!(state.initialized);
    assert_eq!(state.asset, f.asset);
    assert_eq!(state.member_count, 10);
    assert_eq!(state.contribution_amount, CONTRIBUTION);
    assert_eq!(state.minimum_contributions, 3);
    assert_eq!(state.available_pool, 0);
    assert_eq!(state.reserved_pool, 0);
    assert_eq!(state.discount_liability, 0);
    assert_eq!(
        f.client().try_join(&f.first()),
        Err(Ok(Error::MemberAlreadyExists))
    );
    assert_eq!(
        f.client()
            .try_create_community(&f.admin, &f.asset, &CONTRIBUTION, &3, &POT),
        Err(Ok(Error::AlreadyInitialized))
    );
}

#[test]
fn invalid_initialization_and_cycle_membership_are_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let asset = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let contract = env.register(CommunityPool, ());
    let client = CommunityPoolClient::new(&env, &contract);
    assert_eq!(client.try_join(&admin), Err(Ok(Error::NotInitialized)));
    assert_eq!(
        client.try_create_community(&admin, &asset, &0, &3, &POT),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(
        client.try_create_community(&admin, &contract, &CONTRIBUTION, &3, &POT),
        Err(Ok(Error::InvalidAsset))
    );
    env.set_auths(&[]);
    assert!(client
        .try_create_community(&admin, &asset, &CONTRIBUTION, &3, &POT)
        .is_err());
    let f = Fixture::new();
    let outsider = Address::generate(&f.env);
    assert_eq!(
        f.client().try_contribute(&f.first()),
        Err(Ok(Error::CycleNotFound))
    );
    assert_eq!(
        f.client().try_create_cycle(&f.first(), &f.members),
        Err(Ok(Error::Unauthorized))
    );
    assert_eq!(
        f.client()
            .try_create_cycle(&f.admin, &vec![&f.env, f.first(), f.first()]),
        Err(Ok(Error::DuplicateParticipant))
    );
    let cycle = f.start_cycle();
    assert_eq!(cycle, 1);
    assert_eq!(
        f.client().try_create_cycle(&f.admin, &f.members),
        Err(Ok(Error::CycleActive))
    );
    assert_eq!(
        f.client().try_contribute(&outsider),
        Err(Ok(Error::MemberNotFound))
    );
}

#[test]
fn ten_real_contributions_make_exact_fifty_thousand_pot() {
    let f = Fixture::new();
    let cycle = f.start_cycle();
    let token = soroban_sdk::token::TokenClient::new(&f.env, &f.asset);
    let first = f.first();
    let first_before = token.balance(&first);
    f.fund_round();
    assert_eq!(token.balance(&first), first_before - CONTRIBUTION);
    assert_eq!(token.balance(&f.contract), POT);
    let balance = f.client().get_pool_balance();
    assert_eq!(balance.available_pool, POT);
    assert_eq!(balance.reserved_pool, 0);
    assert_eq!(balance.token_balance, POT);
    assert_eq!(f.client().get_member(&first).contributions_completed, 1);
    assert_eq!(
        f.client().get_member(&first).total_contributed,
        CONTRIBUTION
    );
    assert_eq!(
        f.client()
            .get_cycle_member(&cycle, &first)
            .completed_contributions,
        1
    );
    assert_eq!(
        f.client().try_contribute(&first),
        Err(Ok(Error::ContributionAlreadyMade))
    );
    let eligibility = f.client().get_eligibility(&first, &(43_000 * UNIT));
    assert!(eligibility.eligible);
    assert!(eligibility.current_round_contribution_met);
    assert!(eligibility.payout_not_received);
    assert_eq!(eligibility.minimum_contributions, 1);
}

#[test]
fn failed_token_transfer_does_not_advance_accounting() {
    let f = Fixture::new();
    let cycle = f.start_cycle();
    let first = f.first();
    let token = soroban_sdk::token::TokenClient::new(&f.env, &f.asset);
    token.transfer(&first, &MuxedAddress::from(&f.admin), &POT);
    assert!(f.client().try_contribute(&first).is_err());
    assert_eq!(f.client().get_member(&first).contributions_completed, 0);
    assert_eq!(
        f.client()
            .get_cycle_member(&cycle, &first)
            .completed_contributions,
        0
    );
    assert_eq!(f.client().get_pool_balance().available_pool, 0);
}

#[test]
fn eligibility_reasons_and_request_limit_remain_explicit() {
    let f = Fixture::new();
    f.start_cycle();
    let first = f.first();
    let before = f.client().get_eligibility(&first, &(43_000 * UNIT));
    assert!(!before.eligible);
    assert!(!before.current_round_contribution_met);
    f.client().contribute(&first);
    assert!(f.client().get_eligibility(&first, &POT).eligible);
    let over = f.client().get_eligibility(&first, &(POT + 1));
    assert!(!over.eligible);
    assert!(!over.within_financing_limit);
    assert_eq!(
        f.client()
            .try_request_capital(&first, &(POT + 1), &f.purpose()),
        Err(Ok(Error::InvalidAmount))
    );
    let mut state = f.client().get_member(&first);
    state.unresolved_default = true;
    f.env
        .as_contract(&f.contract, || save_member(&f.env, &first, &state));
    assert!(
        !f.client()
            .get_eligibility(&first, &POT)
            .no_unresolved_default
    );
    assert_eq!(
        f.client().try_request_capital(&first, &POT, &f.purpose()),
        Err(Ok(Error::NotEligible))
    );
    state.unresolved_default = false;
    state.active = false;
    f.env
        .as_contract(&f.contract, || save_member(&f.env, &first, &state));
    assert_eq!(
        f.client().try_contribute(&first),
        Err(Ok(Error::InactiveMember))
    );
}

#[test]
fn requests_and_round_reserve_only_current_contributions() {
    let f = Fixture::new();
    let cycle = f.start_cycle();
    let first = f.first();
    assert!(
        !f.client()
            .get_eligibility(&first, &(43_000 * UNIT))
            .eligible
    );
    f.client().contribute(&first);
    let request = f
        .client()
        .request_capital(&first, &(43_000 * UNIT), &f.purpose());
    assert_eq!(f.client().get_request(&request).cycle_id, cycle);
    assert_eq!(
        f.client().get_request(&request).status,
        RequestStatus::Pending
    );
    assert_eq!(
        f.client()
            .try_create_round(&f.admin, &vec![&f.env, request], &100, &200),
        Err(Ok(Error::ContributionsIncomplete))
    );
    for member in f.members.iter() {
        if member != first {
            f.client().contribute(&member);
        }
    }
    let round_id = f
        .client()
        .create_round(&f.admin, &vec![&f.env, request], &100, &200);
    let round = f.client().get_round(&round_id);
    assert_eq!(round.pot, POT);
    assert_eq!(round.available_capital, POT);
    assert_eq!(round.cycle_round_number, 1);
    assert_eq!(round.status, RoundStatus::Commit);
    assert_eq!(
        f.client().get_request(&request).status,
        RequestStatus::IncludedInRound
    );
    assert_eq!(f.client().get_pool_balance().available_pool, 0);
    assert_eq!(f.client().get_pool_balance().reserved_pool, POT);
    assert_eq!(f.client().get_cycle(&cycle).active_round_id, Some(round_id));
}

#[test]
fn invalid_rounds_and_actual_balance_are_checked() {
    let f = Fixture::new();
    f.start_cycle();
    f.fund_round();
    let first = f.first();
    let request = f
        .client()
        .request_capital(&first, &(43_000 * UNIT), &f.purpose());
    assert_eq!(
        f.client()
            .try_create_round(&first, &vec![&f.env, request], &100, &200),
        Err(Ok(Error::Unauthorized))
    );
    assert_eq!(
        f.client()
            .try_create_round(&f.admin, &vec![&f.env, 99], &100, &200),
        Err(Ok(Error::RequestNotFound))
    );
    assert_eq!(
        f.client()
            .try_create_round(&f.admin, &vec![&f.env, request], &100, &100),
        Err(Ok(Error::InvalidDeadline))
    );
    let token = soroban_sdk::token::TokenClient::new(&f.env, &f.asset);
    token.transfer(&f.contract, &MuxedAddress::from(&f.admin), &CONTRIBUTION);
    assert_eq!(
        f.client()
            .try_create_round(&f.admin, &vec![&f.env, request], &100, &200),
        Err(Ok(Error::InsufficientPoolBalance))
    );
    assert_eq!(
        f.client().get_request(&request).status,
        RequestStatus::Pending
    );
}
