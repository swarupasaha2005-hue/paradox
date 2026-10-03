extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, token::StellarAssetClient, vec};

const UNIT: i128 = 10_000_000;
const CONTRIBUTION: i128 = 5_000 * UNIT;
const LIMIT: i128 = 50_000 * UNIT;

struct Fixture {
    env: Env,
    contract: Address,
    admin: Address,
    asset: Address,
    rahul: Address,
    riya: Address,
    aman: Address,
}

impl Fixture {
    fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let rahul = Address::generate(&env);
        let riya = Address::generate(&env);
        let aman = Address::generate(&env);
        let asset = env
            .register_stellar_asset_contract_v2(admin.clone())
            .address();
        let contract = env.register(CommunityPool, ());
        let fixture = Self {
            env,
            contract,
            admin,
            asset,
            rahul,
            riya,
            aman,
        };
        fixture.client().create_community(
            &fixture.admin,
            &fixture.asset,
            &CONTRIBUTION,
            &3,
            &LIMIT,
        );
        let issuer = StellarAssetClient::new(&fixture.env, &fixture.asset);
        for wallet in [&fixture.rahul, &fixture.riya, &fixture.aman] {
            issuer.mint(wallet, &(100_000 * UNIT));
        }
        fixture
    }

    fn client(&self) -> CommunityPoolClient<'_> {
        CommunityPoolClient::new(&self.env, &self.contract)
    }

    fn contribute_n(&self, wallet: &Address, count: usize) {
        self.client().join(wallet);
        for _ in 0..count {
            self.client().contribute(wallet);
        }
    }

    fn purpose(&self) -> BytesN<32> {
        BytesN::from_array(&self.env, &[7; 32])
    }
}

#[test]
fn community_initialized_once_with_real_token() {
    let f = Fixture::new();
    let state = f.client().get_community();
    assert!(state.initialized);
    assert_eq!(state.admin, f.admin);
    assert_eq!(state.asset, f.asset);
    assert_eq!(state.contribution_amount, CONTRIBUTION);
    assert_eq!(state.minimum_contributions, 3);
    assert_eq!(state.financing_limit, LIMIT);
    assert_eq!(state.member_count, 0);
    assert_eq!(state.available_pool, 0);
    assert_eq!(state.request_counter, 0);
    assert_eq!(state.round_counter, 0);
    assert_eq!(
        f.client()
            .try_create_community(&f.admin, &f.asset, &CONTRIBUTION, &3, &LIMIT),
        Err(Ok(Error::AlreadyInitialized))
    );
}

#[test]
fn rejects_invalid_initialization_and_unauthorized_wallet_actions() {
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
        client.try_create_community(&admin, &asset, &0, &3, &LIMIT),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(
        client.try_create_community(&admin, &contract, &CONTRIBUTION, &3, &LIMIT),
        Err(Ok(Error::InvalidAsset))
    );
    env.set_auths(&[]);
    assert!(client
        .try_create_community(&admin, &asset, &CONTRIBUTION, &3, &LIMIT)
        .is_err());
    env.mock_all_auths();
    client.create_community(&admin, &asset, &CONTRIBUTION, &3, &LIMIT);
    env.set_auths(&[]);
    assert!(client.try_join(&admin).is_err());

    env.mock_all_auths();
    client.join(&admin);
    env.set_auths(&[]);
    assert!(client.try_contribute(&admin).is_err());
    assert!(client
        .try_request_capital(&admin, &LIMIT, &BytesN::from_array(&env, &[0; 32]))
        .is_err());
}

#[test]
fn join_and_real_contribution_update_balances_and_history() {
    let f = Fixture::new();
    let client = f.client();
    assert!(!client.is_member(&f.rahul));
    client.join(&f.rahul);
    assert!(client.is_member(&f.rahul));
    assert_eq!(client.get_community().member_count, 1);
    assert_eq!(
        client.try_join(&f.rahul),
        Err(Ok(Error::MemberAlreadyExists))
    );
    assert_eq!(
        client.try_contribute(&f.riya),
        Err(Ok(Error::MemberNotFound))
    );

    let token = TokenClient::new(&f.env, &f.asset);
    assert_eq!(token.decimals(), 7);
    let before_member = token.balance(&f.rahul);
    let before_pool = token.balance(&f.contract);
    client.contribute(&f.rahul);
    assert_eq!(token.balance(&f.rahul), before_member - CONTRIBUTION);
    assert_eq!(token.balance(&f.contract), before_pool + CONTRIBUTION);
    let balance = client.get_pool_balance();
    assert_eq!(balance.available_pool, CONTRIBUTION);
    assert_eq!(balance.token_balance, CONTRIBUTION);
    let history = client.get_member(&f.rahul);
    assert_eq!(history.contributions_completed, 1);
    assert_eq!(history.total_contributed, CONTRIBUTION);
    assert_eq!(history.contributions_missed, 0);
    assert_eq!(history.repayments_missed, 0);
    assert!(!history.unresolved_default);
}

#[test]
fn failed_token_transfer_does_not_advance_accounting() {
    let f = Fixture::new();
    let unfunded = Address::generate(&f.env);
    let client = f.client();
    client.join(&unfunded);
    assert!(client.try_contribute(&unfunded).is_err());
    assert_eq!(client.get_member(&unfunded).contributions_completed, 0);
    assert_eq!(client.get_member(&unfunded).total_contributed, 0);
    assert_eq!(client.get_pool_balance().available_pool, 0);
    assert_eq!(client.get_pool_balance().token_balance, 0);
}

#[test]
fn eligibility_flags_explain_inactive_default_and_outstanding() {
    let f = Fixture::new();
    f.contribute_n(&f.rahul, 3);
    let amount = 43_000 * UNIT;
    let mut member = f.client().get_member(&f.rahul);
    member.active = false;
    f.env
        .as_contract(&f.contract, || save_member(&f.env, &f.rahul, &member));
    let result = f.client().get_eligibility(&f.rahul, &amount);
    assert!(!result.eligible);
    assert!(!result.active);
    assert_eq!(
        f.client().try_contribute(&f.rahul),
        Err(Ok(Error::InactiveMember))
    );
    assert_eq!(
        f.client()
            .try_request_capital(&f.rahul, &amount, &f.purpose()),
        Err(Ok(Error::InactiveMember))
    );

    member.active = true;
    member.unresolved_default = true;
    f.env
        .as_contract(&f.contract, || save_member(&f.env, &f.rahul, &member));
    let result = f.client().get_eligibility(&f.rahul, &amount);
    assert!(!result.eligible);
    assert!(!result.no_unresolved_default);
    assert_eq!(
        f.client()
            .try_request_capital(&f.rahul, &amount, &f.purpose()),
        Err(Ok(Error::NotEligible))
    );

    member.unresolved_default = false;
    member.outstanding_financing = 1;
    f.env
        .as_contract(&f.contract, || save_member(&f.env, &f.rahul, &member));
    let result = f.client().get_eligibility(&f.rahul, &amount);
    assert!(!result.eligible);
    assert!(!result.no_outstanding_financing);
    assert_eq!(
        f.client()
            .try_request_capital(&f.rahul, &amount, &f.purpose()),
        Err(Ok(Error::NotEligible))
    );
}

#[test]
fn three_contributions_and_financing_limit_control_eligibility() {
    let f = Fixture::new();
    let client = f.client();
    let amount = 43_000 * UNIT;
    let missing = client.get_eligibility(&f.rahul, &amount);
    assert!(!missing.eligible);
    assert!(!missing.is_member);
    assert_eq!(missing.minimum_contributions, 3);
    client.join(&f.rahul);
    for completed in 0..3 {
        let state = client.get_eligibility(&f.rahul, &amount);
        assert!(!state.eligible);
        assert!(!state.contribution_requirement_met);
        assert_eq!(state.contributions_completed, completed);
        assert_eq!(
            client.try_request_capital(&f.rahul, &amount, &f.purpose()),
            Err(Ok(Error::NotEligible))
        );
        client.contribute(&f.rahul);
    }
    assert!(client.get_eligibility(&f.rahul, &amount).eligible);
    assert!(client.get_eligibility(&f.rahul, &LIMIT).eligible);
    let over_limit = client.get_eligibility(&f.rahul, &(LIMIT + 1));
    assert!(!over_limit.eligible);
    assert!(!over_limit.within_financing_limit);
    assert_eq!(
        client.try_request_capital(&f.rahul, &(LIMIT + 1), &f.purpose()),
        Err(Ok(Error::InvalidAmount))
    );
    let zero = client.get_eligibility(&f.rahul, &0);
    assert!(!zero.eligible);
    assert!(!zero.valid_amount);
}

#[test]
fn eligible_capital_request_is_pending_and_indexed() {
    let f = Fixture::new();
    f.contribute_n(&f.rahul, 3);
    let id = f
        .client()
        .request_capital(&f.rahul, &(43_000 * UNIT), &f.purpose());
    assert_eq!(id, 1);
    let request = f.client().get_request(&id);
    assert_eq!(request.member, f.rahul);
    assert_eq!(request.maximum_amount, 43_000 * UNIT);
    assert_eq!(request.purpose_hash, f.purpose());
    assert_eq!(request.status, RequestStatus::Pending);
    assert_eq!(f.client().get_community().request_counter, 1);
    assert_eq!(
        f.client().try_get_request(&2),
        Err(Ok(Error::RequestNotFound))
    );
}

#[test]
fn round_caps_capital_below_request_maximum() {
    let f = Fixture::new();
    f.contribute_n(&f.rahul, 3);
    let client = f.client();
    let request = client.request_capital(&f.rahul, &(43_000 * UNIT), &f.purpose());
    let round_id = client.create_round(&f.admin, &vec![&f.env, request], &100, &200);
    assert_eq!(client.get_round(&round_id).available_capital, 15_000 * UNIT);
    assert_eq!(
        client.get_request(&request).status,
        RequestStatus::IncludedInRound
    );
}

#[test]
fn round_cap_uses_actual_token_balance_when_lower_than_accounting() {
    let f = Fixture::new();
    f.contribute_n(&f.rahul, 3);
    let client = f.client();
    let token = TokenClient::new(&f.env, &f.asset);
    // Simulate a prior outflow before settlement accounting exists in Step 3.
    token.transfer(&f.contract, &MuxedAddress::from(&f.admin), &(10_000 * UNIT));
    let balance = client.get_pool_balance();
    assert_eq!(balance.available_pool, 15_000 * UNIT);
    assert_eq!(balance.token_balance, 5_000 * UNIT);
    let request = client.request_capital(&f.rahul, &(43_000 * UNIT), &f.purpose());
    let round = client.create_round(&f.admin, &vec![&f.env, request], &100, &200);
    assert_eq!(client.get_round(&round).available_capital, 5_000 * UNIT);

    token.transfer(&f.contract, &MuxedAddress::from(&f.admin), &(5_000 * UNIT));
    let next = client.request_capital(&f.rahul, &(43_000 * UNIT), &f.purpose());
    assert_eq!(
        client.try_create_round(&f.admin, &vec![&f.env, next], &100, &200),
        Err(Ok(Error::InsufficientPoolBalance))
    );
    assert_eq!(client.get_request(&next).status, RequestStatus::Pending);
}

#[test]
fn three_member_round_has_fifty_thousand_available_and_marks_requests() {
    let f = Fixture::new();
    f.contribute_n(&f.rahul, 4);
    f.contribute_n(&f.riya, 3);
    f.contribute_n(&f.aman, 3);
    let client = f.client();
    assert_eq!(client.get_pool_balance().available_pool, 50_000 * UNIT);
    assert_eq!(client.get_pool_balance().token_balance, 50_000 * UNIT);
    assert_eq!(client.get_community().member_count, 3);
    let r1 = client.request_capital(&f.rahul, &(43_000 * UNIT), &f.purpose());
    let r2 = client.request_capital(&f.riya, &(46_000 * UNIT), &f.purpose());
    let r3 = client.request_capital(&f.aman, &(45_000 * UNIT), &f.purpose());
    let id = client.create_round(&f.admin, &vec![&f.env, r1, r2, r3], &100, &200);
    assert_eq!(id, 1);
    let round = client.get_round(&id);
    assert_eq!(round.request_ids, vec![&f.env, r1, r2, r3]);
    assert_eq!(
        round.participants,
        vec![&f.env, f.rahul.clone(), f.riya.clone(), f.aman.clone()]
    );
    assert_eq!(round.available_capital, 50_000 * UNIT);
    assert_eq!(round.commit_deadline, 100);
    assert_eq!(round.reveal_deadline, 200);
    assert_eq!(round.status, RoundStatus::Commit);
    assert!(round.winner.is_none());
    assert!(round.winning_bid.is_none());
    assert!(!round.settled);
    for request_id in [r1, r2, r3] {
        assert_eq!(
            client.get_request(&request_id).status,
            RequestStatus::IncludedInRound
        );
    }
    assert_eq!(client.get_community().round_counter, 1);
}

#[test]
fn invalid_rounds_are_rejected_without_changing_requests() {
    let f = Fixture::new();
    f.contribute_n(&f.rahul, 5);
    f.contribute_n(&f.riya, 3);
    let client = f.client();
    let first = client.request_capital(&f.rahul, &(40_000 * UNIT), &f.purpose());
    let second = client.request_capital(&f.riya, &(40_000 * UNIT), &f.purpose());
    assert_eq!(
        client.try_create_round(&f.rahul, &vec![&f.env, first], &100, &200),
        Err(Ok(Error::Unauthorized))
    );
    assert_eq!(
        client.try_create_round(&f.admin, &vec![&f.env, 99], &100, &200),
        Err(Ok(Error::RequestNotFound))
    );
    assert_eq!(
        client.try_create_round(&f.admin, &vec![&f.env, first], &0, &200),
        Err(Ok(Error::InvalidDeadline))
    );
    assert_eq!(
        client.try_create_round(&f.admin, &vec![&f.env, first], &100, &100),
        Err(Ok(Error::InvalidDeadline))
    );
    assert_eq!(
        client.try_create_round(&f.admin, &vec![&f.env, first, first], &100, &200),
        Err(Ok(Error::DuplicateParticipant))
    );
    assert_eq!(client.get_request(&first).status, RequestStatus::Pending);
    client.create_round(&f.admin, &vec![&f.env, first], &100, &200);
    assert_eq!(
        client.try_create_round(&f.admin, &vec![&f.env, first, second], &100, &200),
        Err(Ok(Error::InvalidRequestStatus))
    );
    assert_eq!(client.get_request(&second).status, RequestStatus::Pending);
}
