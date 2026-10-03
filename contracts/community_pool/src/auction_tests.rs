extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    vec,
};

const UNIT: i128 = 10_000_000;
const BID_RAHUL: i128 = 43_000 * UNIT;
const BID_RIYA: i128 = 46_000 * UNIT;
const BID_AMAN: i128 = 45_000 * UNIT;

struct AuctionFixture {
    env: Env,
    contract: Address,
    rahul: Address,
    riya: Address,
    aman: Address,
    outsider: Address,
    round_id: u64,
}

impl AuctionFixture {
    fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let rahul = Address::generate(&env);
        let riya = Address::generate(&env);
        let aman = Address::generate(&env);
        let outsider = Address::generate(&env);
        let asset = env
            .register_stellar_asset_contract_v2(admin.clone())
            .address();
        let contract = env.register(CommunityPool, ());
        let client = CommunityPoolClient::new(&env, &contract);
        let contribution = 5_000 * UNIT;
        client.create_community(&admin, &asset, &contribution, &3, &(50_000 * UNIT));
        let token = StellarAssetClient::new(&env, &asset);
        for (participant, count) in [(&rahul, 4), (&riya, 3), (&aman, 3)] {
            token.mint(participant, &(100_000 * UNIT));
            client.join(participant);
            for _ in 0..count {
                client.contribute(participant);
            }
        }
        let purpose = BytesN::from_array(&env, &[7; 32]);
        let rahul_request = client.request_capital(&rahul, &BID_RAHUL, &purpose);
        let riya_request = client.request_capital(&riya, &BID_RIYA, &purpose);
        let aman_request = client.request_capital(&aman, &BID_AMAN, &purpose);
        let round_id = client.create_round(
            &admin,
            &vec![&env, rahul_request, riya_request, aman_request],
            &100,
            &200,
        );
        Self {
            env,
            contract,
            rahul,
            riya,
            aman,
            outsider,
            round_id,
        }
    }

    fn client(&self) -> CommunityPoolClient<'_> {
        CommunityPoolClient::new(&self.env, &self.contract)
    }

    fn secret(&self, marker: u8) -> BytesN<32> {
        BytesN::from_array(&self.env, &[marker; 32])
    }

    fn commitment(&self, participant: &Address, bid: i128, secret: &BytesN<32>) -> BytesN<32> {
        commitment::hash(&self.env, self.round_id, participant, bid, secret)
    }

    fn commit(&self, participant: &Address, bid: i128, secret: &BytesN<32>) {
        self.client().commit_bid(
            &self.round_id,
            participant,
            &self.commitment(participant, bid, secret),
        );
    }

    fn open_reveal(&self) {
        self.env.ledger().set_timestamp(100);
        self.client().start_reveal(&self.round_id);
    }

    fn finalized_with_rahul_winning() -> Self {
        let f = Self::new();
        let rahul_secret = f.secret(121);
        let riya_secret = f.secret(122);
        let aman_secret = f.secret(123);
        f.commit(&f.rahul, BID_RAHUL, &rahul_secret);
        f.commit(&f.riya, BID_RIYA, &riya_secret);
        f.commit(&f.aman, BID_AMAN, &aman_secret);
        f.open_reveal();
        f.client()
            .reveal_bid(&f.round_id, &f.rahul, &BID_RAHUL, &rahul_secret);
        f.client()
            .reveal_bid(&f.round_id, &f.riya, &BID_RIYA, &riya_secret);
        f.client()
            .reveal_bid(&f.round_id, &f.aman, &BID_AMAN, &aman_secret);
        f.env.ledger().set_timestamp(200);
        f.client().finalize_round(&f.round_id);
        f
    }
}

#[test]
fn correct_reveal_passes_and_wrong_amount_or_secret_is_rejected() {
    let f = AuctionFixture::new();
    let secret = f.secret(11);
    f.commit(&f.rahul, BID_RAHUL, &secret);
    f.open_reveal();
    assert_eq!(
        f.client()
            .try_reveal_bid(&f.round_id, &f.rahul, &(42_000 * UNIT), &secret),
        Err(Ok(Error::CommitmentMismatch))
    );
    assert_eq!(
        f.client()
            .try_reveal_bid(&f.round_id, &f.rahul, &BID_RAHUL, &f.secret(12)),
        Err(Ok(Error::CommitmentMismatch))
    );
    assert_eq!(f.client().get_reveal(&f.round_id, &f.rahul), None);
    f.client()
        .reveal_bid(&f.round_id, &f.rahul, &BID_RAHUL, &secret);
    assert_eq!(
        f.client().get_reveal(&f.round_id, &f.rahul),
        Some(BID_RAHUL)
    );
    assert_eq!(f.client().get_round(&f.round_id).valid_reveal_count, 1);
}

#[test]
fn commit_phase_stores_hash_only_and_rejects_second_commitment() {
    let f = AuctionFixture::new();
    let secret = f.secret(21);
    let hash = f.commitment(&f.rahul, BID_RAHUL, &secret);
    f.client().commit_bid(&f.round_id, &f.rahul, &hash);
    assert_eq!(
        f.client().get_commitment(&f.round_id, &f.rahul),
        Some(hash.clone())
    );
    assert_eq!(f.client().get_reveal(&f.round_id, &f.rahul), None);
    let round = f.client().get_round(&f.round_id);
    assert_eq!(round.status, RoundStatus::Commit);
    assert_eq!(round.valid_reveal_count, 0);
    assert!(round.winning_bid.is_none());
    assert!(round.winner.is_none());
    assert_eq!(
        f.client().try_commit_bid(
            &f.round_id,
            &f.rahul,
            &f.commitment(&f.rahul, 42_000 * UNIT, &secret)
        ),
        Err(Ok(Error::CommitmentAlreadyExists))
    );
    assert_eq!(f.client().get_commitment(&f.round_id, &f.rahul), Some(hash));
}

#[test]
fn nonparticipant_and_unauthorized_wallet_cannot_commit_or_reveal() {
    let f = AuctionFixture::new();
    let secret = f.secret(31);
    let outsider_hash = f.commitment(&f.outsider, BID_RAHUL, &secret);
    assert_eq!(
        f.client()
            .try_commit_bid(&f.round_id, &f.outsider, &outsider_hash),
        Err(Ok(Error::NotParticipant))
    );
    f.commit(&f.rahul, BID_RAHUL, &secret);
    f.env.set_auths(&[]);
    assert!(f
        .client()
        .try_commit_bid(&f.round_id, &f.riya, &outsider_hash)
        .is_err());
    f.env.mock_all_auths();
    f.open_reveal();
    assert_eq!(
        f.client()
            .try_reveal_bid(&f.round_id, &f.outsider, &BID_RAHUL, &secret),
        Err(Ok(Error::NotParticipant))
    );
    f.env.set_auths(&[]);
    assert!(f
        .client()
        .try_reveal_bid(&f.round_id, &f.rahul, &BID_RAHUL, &secret)
        .is_err());
}

#[test]
fn deadlines_and_phase_transition_are_enforced() {
    let f = AuctionFixture::new();
    let secret = f.secret(41);
    f.commit(&f.rahul, BID_RAHUL, &secret);
    assert_eq!(
        f.client()
            .try_reveal_bid(&f.round_id, &f.rahul, &BID_RAHUL, &secret),
        Err(Ok(Error::RevealNotOpen))
    );
    assert_eq!(
        f.client().try_start_reveal(&f.round_id),
        Err(Ok(Error::CommitClosed))
    );
    f.env.ledger().set_timestamp(100);
    assert_eq!(
        f.client().try_commit_bid(
            &f.round_id,
            &f.riya,
            &f.commitment(&f.riya, BID_RIYA, &secret)
        ),
        Err(Ok(Error::CommitClosed))
    );
    f.client().start_reveal(&f.round_id);
    assert_eq!(
        f.client().get_round(&f.round_id).status,
        RoundStatus::Reveal
    );
    assert_eq!(
        f.client().try_start_reveal(&f.round_id),
        Err(Ok(Error::InvalidRoundStatus))
    );
    f.env.ledger().set_timestamp(200);
    assert_eq!(
        f.client()
            .try_reveal_bid(&f.round_id, &f.rahul, &BID_RAHUL, &secret),
        Err(Ok(Error::RevealClosed))
    );
}

#[test]
fn duplicate_reveal_and_post_reveal_recommit_are_rejected() {
    let f = AuctionFixture::new();
    let rahul_secret = f.secret(51);
    let riya_secret = f.secret(52);
    f.commit(&f.rahul, BID_RAHUL, &rahul_secret);
    f.commit(&f.riya, BID_RIYA, &riya_secret);
    let original_hash = f.client().get_commitment(&f.round_id, &f.rahul);
    f.open_reveal();
    f.client()
        .reveal_bid(&f.round_id, &f.riya, &BID_RIYA, &riya_secret);
    assert_eq!(
        f.client().try_commit_bid(
            &f.round_id,
            &f.rahul,
            &f.commitment(&f.rahul, 42_000 * UNIT, &rahul_secret)
        ),
        Err(Ok(Error::InvalidRoundStatus))
    );
    assert_eq!(
        f.client().get_commitment(&f.round_id, &f.rahul),
        original_hash
    );
    f.client()
        .reveal_bid(&f.round_id, &f.rahul, &BID_RAHUL, &rahul_secret);
    assert_eq!(
        f.client()
            .try_reveal_bid(&f.round_id, &f.rahul, &BID_RAHUL, &rahul_secret),
        Err(Ok(Error::RevealAlreadyExists))
    );
    assert_eq!(f.client().get_round(&f.round_id).valid_reveal_count, 2);
}

#[test]
fn invalid_bid_bounds_and_missing_commitment_are_rejected() {
    let f = AuctionFixture::new();
    let secret = f.secret(61);
    f.commit(&f.rahul, BID_RAHUL, &secret);
    f.open_reveal();
    assert_eq!(
        f.client()
            .try_reveal_bid(&f.round_id, &f.riya, &BID_RIYA, &secret),
        Err(Ok(Error::CommitmentNotFound))
    );
    assert_eq!(
        f.client()
            .try_reveal_bid(&f.round_id, &f.rahul, &0, &secret),
        Err(Ok(Error::InvalidBid))
    );
    assert_eq!(
        f.client()
            .try_reveal_bid(&f.round_id, &f.rahul, &(BID_RAHUL + 1), &secret),
        Err(Ok(Error::InvalidBid))
    );
}

#[test]
fn lowest_valid_reveal_wins_unrevealed_participant_cannot_win() {
    let f = AuctionFixture::new();
    let s1 = f.secret(71);
    let s2 = f.secret(72);
    let s3 = f.secret(73);
    f.commit(&f.rahul, BID_RAHUL, &s1);
    f.commit(&f.riya, BID_RIYA, &s2);
    f.commit(&f.aman, BID_AMAN, &s3);
    f.open_reveal();
    f.client()
        .reveal_bid(&f.round_id, &f.rahul, &BID_RAHUL, &s1);
    f.client().reveal_bid(&f.round_id, &f.riya, &BID_RIYA, &s2);
    f.client().reveal_bid(&f.round_id, &f.aman, &BID_AMAN, &s3);
    assert_eq!(
        f.client().try_finalize_round(&f.round_id),
        Err(Ok(Error::FinalizationTooEarly))
    );
    f.env.ledger().set_timestamp(200);
    f.client().finalize_round(&f.round_id);
    let round = f.client().get_round(&f.round_id);
    assert_eq!(round.status, RoundStatus::Finalized);
    assert_eq!(round.winner, Some(f.rahul.clone()));
    assert_eq!(round.winning_bid, Some(BID_RAHUL));
    assert_eq!(round.valid_reveal_count, 3);
    assert_eq!(
        f.client().try_finalize_round(&f.round_id),
        Err(Ok(Error::InvalidRoundStatus))
    );
}

#[test]
fn tie_breaks_by_ascending_wallet_strkey_not_reveal_order() {
    let f = AuctionFixture::new();
    let amount = 40_000 * UNIT;
    let s1 = f.secret(81);
    let s2 = f.secret(82);
    let s3 = f.secret(83);
    f.commit(&f.rahul, amount, &s1);
    f.commit(&f.riya, amount, &s2);
    f.commit(&f.aman, amount, &s3);
    f.open_reveal();
    f.client().reveal_bid(&f.round_id, &f.aman, &amount, &s3);
    f.client().reveal_bid(&f.round_id, &f.riya, &amount, &s2);
    f.client().reveal_bid(&f.round_id, &f.rahul, &amount, &s1);
    f.env.ledger().set_timestamp(200);
    f.client().finalize_round(&f.round_id);
    let mut expected = f.rahul.clone();
    for candidate in [&f.riya, &f.aman] {
        if candidate.to_string() < expected.to_string() {
            expected = candidate.clone();
        }
    }
    assert_eq!(f.client().get_round(&f.round_id).winner, Some(expected));
    assert_eq!(f.client().get_round(&f.round_id).winning_bid, Some(amount));
}

#[test]
fn no_valid_reveal_finalizes_without_a_winner() {
    let f = AuctionFixture::new();
    let secret = f.secret(91);
    f.commit(&f.rahul, BID_RAHUL, &secret);
    f.open_reveal();
    f.env.ledger().set_timestamp(200);
    f.client().finalize_round(&f.round_id);
    let round = f.client().get_round(&f.round_id);
    assert_eq!(round.status, RoundStatus::Finalized);
    assert_eq!(round.valid_reveal_count, 0);
    assert!(round.winner.is_none());
    assert!(round.winning_bid.is_none());
    assert_eq!(f.client().get_reveal(&f.round_id, &f.rahul), None);
    assert_eq!(
        f.client().try_settle(&f.round_id, &f.rahul, &300),
        Err(Ok(Error::NoWinner))
    );
}

#[test]
fn unrevealed_lower_commitment_cannot_beat_a_valid_reveal() {
    let f = AuctionFixture::new();
    let rahul_secret = f.secret(101);
    let riya_secret = f.secret(102);
    f.commit(&f.rahul, BID_RAHUL, &rahul_secret);
    f.commit(&f.riya, BID_RIYA, &riya_secret);
    f.open_reveal();
    f.client()
        .reveal_bid(&f.round_id, &f.riya, &BID_RIYA, &riya_secret);
    f.env.ledger().set_timestamp(200);
    f.client().finalize_round(&f.round_id);
    let round = f.client().get_round(&f.round_id);
    assert_eq!(round.winner, Some(f.riya.clone()));
    assert_eq!(round.winning_bid, Some(BID_RIYA));
    assert_eq!(round.valid_reveal_count, 1);
    assert_eq!(f.client().get_reveal(&f.round_id, &f.rahul), None);
}

#[test]
fn participant_must_remain_eligible_to_commit() {
    let f = AuctionFixture::new();
    let mut member = f.client().get_member(&f.rahul);
    member.unresolved_default = true;
    f.env
        .as_contract(&f.contract, || save_member(&f.env, &f.rahul, &member));
    let secret = f.secret(111);
    assert_eq!(
        f.client().try_commit_bid(
            &f.round_id,
            &f.rahul,
            &f.commitment(&f.rahul, BID_RAHUL, &secret),
        ),
        Err(Ok(Error::NotEligible))
    );
    assert_eq!(f.client().get_commitment(&f.round_id, &f.rahul), None);
}

#[test]
fn reveal_cannot_exceed_round_capital_snapshot() {
    let f = AuctionFixture::new();
    let community = f.client().get_community();
    let token = soroban_sdk::token::TokenClient::new(&f.env, &community.asset);
    token.transfer(
        &f.contract,
        &MuxedAddress::from(&f.outsider),
        &(20_000 * UNIT),
    );
    let purpose = BytesN::from_array(&f.env, &[8; 32]);
    let request = f.client().request_capital(&f.rahul, &BID_RAHUL, &purpose);
    let second_round =
        f.client()
            .create_round(&community.admin, &vec![&f.env, request], &100, &200);
    assert_eq!(
        f.client().get_round(&second_round).available_capital,
        30_000 * UNIT
    );
    let secret = f.secret(112);
    let hash = commitment::hash(&f.env, second_round, &f.rahul, BID_RAHUL, &secret);
    f.client().commit_bid(&second_round, &f.rahul, &hash);
    f.env.ledger().set_timestamp(100);
    f.client().start_reveal(&second_round);
    assert_eq!(
        f.client()
            .try_reveal_bid(&second_round, &f.rahul, &BID_RAHUL, &secret),
        Err(Ok(Error::InsufficientPoolBalance))
    );
    assert_eq!(f.client().get_reveal(&second_round, &f.rahul), None);
}

#[test]
fn settlement_transfers_once_and_full_repayment_restores_pool() {
    let f = AuctionFixture::finalized_with_rahul_winning();
    let client = f.client();
    let asset = client.get_community().asset;
    let token = soroban_sdk::token::TokenClient::new(&f.env, &asset);
    let rahul_before = token.balance(&f.rahul);
    client.settle(&f.round_id, &f.rahul, &300);
    assert_eq!(token.balance(&f.rahul), rahul_before + BID_RAHUL);
    assert_eq!(token.balance(&f.contract), 7_000 * UNIT);
    assert_eq!(client.get_pool_balance().available_pool, 7_000 * UNIT);
    let round = client.get_round(&f.round_id);
    assert_eq!(round.status, RoundStatus::Settled);
    assert!(round.settled);
    assert_eq!(round.outstanding_amount, BID_RAHUL);
    assert_eq!(round.due_date, Some(300));
    assert_eq!(client.get_member(&f.rahul).outstanding_financing, BID_RAHUL);
    assert_eq!(client.get_request(&1).status, RequestStatus::Funded);
    assert_eq!(
        client.try_settle(&f.round_id, &f.rahul, &300),
        Err(Ok(Error::AlreadySettled))
    );

    client.repay(&f.round_id, &f.rahul, &BID_RAHUL);
    assert_eq!(token.balance(&f.rahul), rahul_before);
    assert_eq!(token.balance(&f.contract), 50_000 * UNIT);
    assert_eq!(client.get_pool_balance().available_pool, 50_000 * UNIT);
    assert_eq!(client.get_round(&f.round_id).outstanding_amount, 0);
    let member = client.get_member(&f.rahul);
    assert_eq!(member.outstanding_financing, 0);
    assert_eq!(member.financing_rounds_completed, 1);
    assert_eq!(member.repayments_completed, 1);
    assert_eq!(member.repayments_missed, 0);
    assert_eq!(client.get_request(&1).status, RequestStatus::Repaid);
    assert_eq!(
        client.try_repay(&f.round_id, &f.rahul, &1),
        Err(Ok(Error::InvalidRepayment))
    );
}

#[test]
fn invalid_winner_deadline_and_insufficient_pool_block_settlement() {
    let f = AuctionFixture::finalized_with_rahul_winning();
    let client = f.client();
    assert_eq!(
        client.try_settle(&f.round_id, &f.riya, &300),
        Err(Ok(Error::NoWinner))
    );
    assert_eq!(
        client.try_settle(&f.round_id, &f.rahul, &200),
        Err(Ok(Error::InvalidDeadline))
    );
    let asset = client.get_community().asset;
    let token = soroban_sdk::token::TokenClient::new(&f.env, &asset);
    token.transfer(
        &f.contract,
        &MuxedAddress::from(&f.outsider),
        &(8_000 * UNIT),
    );
    assert_eq!(
        client.try_settle(&f.round_id, &f.rahul, &300),
        Err(Ok(Error::InsufficientPoolBalance))
    );
    assert_eq!(client.get_round(&f.round_id).status, RoundStatus::Finalized);
    assert_eq!(client.get_community().available_pool, 50_000 * UNIT);
    assert_eq!(client.get_member(&f.rahul).outstanding_financing, 0);
}

#[test]
fn unauthorized_or_excess_repayment_fails_and_partial_repayment_tracks_balance() {
    let f = AuctionFixture::finalized_with_rahul_winning();
    let client = f.client();
    client.settle(&f.round_id, &f.rahul, &300);
    f.env.set_auths(&[]);
    assert!(client
        .try_repay(&f.round_id, &f.rahul, &(1_000 * UNIT))
        .is_err());
    f.env.mock_all_auths();
    assert_eq!(
        client.try_repay(&f.round_id, &f.rahul, &(BID_RAHUL + 1)),
        Err(Ok(Error::InvalidRepayment))
    );
    assert_eq!(
        client.try_repay(&f.round_id, &f.riya, &(1_000 * UNIT)),
        Err(Ok(Error::InvalidRoundStatus))
    );
    client.repay(&f.round_id, &f.rahul, &(3_000 * UNIT));
    assert_eq!(
        client.get_round(&f.round_id).outstanding_amount,
        40_000 * UNIT
    );
    assert_eq!(client.get_member(&f.rahul).repayments_completed, 0);
    assert_eq!(client.get_request(&1).status, RequestStatus::Funded);
    client.repay(&f.round_id, &f.rahul, &(40_000 * UNIT));
    assert_eq!(client.get_member(&f.rahul).repayments_completed, 1);
    assert_eq!(client.get_request(&1).status, RequestStatus::Repaid);
}
