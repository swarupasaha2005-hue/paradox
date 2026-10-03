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
    admin: Address,
    rahul: Address,
    riya: Address,
    aman: Address,
    outsider: Address,
    cycle_id: u64,
    members: Vec<Address>,
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
        let mut members = vec![&env, rahul.clone(), riya.clone(), aman.clone()];
        for _ in 3..10 {
            members.push_back(Address::generate(&env));
        }
        for participant in members.iter() {
            token.mint(&participant, &(100_000 * UNIT));
            client.join(&participant);
        }
        let cycle_id = client.create_cycle(&admin, &members);
        for participant in members.iter() {
            client.contribute(&participant);
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
            admin,
            rahul,
            riya,
            aman,
            outsider,
            cycle_id,
            members,
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
        f.client().try_settle(&f.round_id, &f.rahul),
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
fn one_round_reserves_its_pot_and_cannot_be_reused() {
    let f = AuctionFixture::new();
    let balance = f.client().get_pool_balance();
    assert_eq!(balance.available_pool, 0);
    assert_eq!(balance.reserved_pool, 50_000 * UNIT);
    assert_eq!(balance.discount_liability, 0);
    assert_eq!(balance.token_balance, 50_000 * UNIT);
    assert_eq!(
        f.client()
            .try_create_round(&f.admin, &vec![&f.env, 1], &100, &200),
        Err(Ok(Error::RoundAlreadyActive))
    );
    assert_eq!(f.client().get_round(&f.round_id).pot, 50_000 * UNIT);
}

#[test]
fn settlement_pays_discounted_payout_and_credits_all_ten_members() {
    let f = AuctionFixture::finalized_with_rahul_winning();
    let client = f.client();
    let asset = client.get_community().asset;
    let token = soroban_sdk::token::TokenClient::new(&f.env, &asset);
    let rahul_before = token.balance(&f.rahul);
    client.settle(&f.round_id, &f.rahul);
    assert_eq!(token.balance(&f.rahul), rahul_before + BID_RAHUL);
    assert_eq!(token.balance(&f.contract), 7_000 * UNIT);
    let balance = client.get_pool_balance();
    assert_eq!(balance.available_pool, 0);
    assert_eq!(balance.reserved_pool, 0);
    assert_eq!(balance.discount_liability, 7_000 * UNIT);
    assert_eq!(balance.token_balance, balance.discount_liability);
    let round = client.get_round(&f.round_id);
    assert_eq!(round.status, RoundStatus::Settled);
    assert_eq!(round.auction_discount, 7_000 * UNIT);
    assert_eq!(round.discount_per_member, 700 * UNIT);
    assert_eq!(round.discount_remainder, 0);
    assert_eq!(
        round.winning_bid.unwrap() + round.auction_discount,
        round.pot
    );
    assert_eq!(client.get_request(&1).status, RequestStatus::PaidOut);
    assert_eq!(client.get_request(&2).status, RequestStatus::NotSelected);
    assert_eq!(
        client.get_member(&f.rahul).total_payouts_received,
        BID_RAHUL
    );
    assert_eq!(client.get_member(&f.rahul).payouts_received, 1);
    let mut sum = 0;
    for address in f.members.iter() {
        let state = client.get_cycle_member(&f.cycle_id, &address);
        assert_eq!(state.claimable_discount, 700 * UNIT);
        sum += state.claimable_discount;
    }
    assert_eq!(sum, round.auction_discount);
    assert!(
        client
            .get_cycle_member(&f.cycle_id, &f.rahul)
            .payout_received
    );
    assert_eq!(
        client.try_settle(&f.round_id, &f.rahul),
        Err(Ok(Error::AlreadySettled))
    );
}

#[test]
fn winner_continues_contributing_and_cannot_win_twice_in_cycle() {
    let f = AuctionFixture::finalized_with_rahul_winning();
    let client = f.client();
    client.settle(&f.round_id, &f.rahul);
    let before = client.get_cycle_member(&f.cycle_id, &f.rahul);
    assert_eq!(before.expected_contributions, 2);
    assert_eq!(before.completed_contributions, 1);
    assert_eq!(before.contributions_before_payout, 1);
    assert_eq!(before.contributions_after_payout, 0);
    assert!(!before.obligation_complete);
    assert_eq!(
        client.try_request_capital(&f.rahul, &BID_RAHUL, &BytesN::from_array(&f.env, &[9; 32])),
        Err(Ok(Error::NotEligible))
    );
    client.contribute(&f.rahul);
    let after = client.get_cycle_member(&f.cycle_id, &f.rahul);
    assert_eq!(after.completed_contributions, 2);
    assert_eq!(after.contributions_after_payout, 1);
    assert!(
        !client
            .get_eligibility(&f.rahul, &BID_RAHUL)
            .payout_not_received
    );
    assert_eq!(
        client.try_contribute(&f.rahul),
        Err(Ok(Error::ContributionAlreadyMade))
    );
    for address in f.members.iter() {
        if address != f.rahul {
            client.contribute(&address);
        }
    }
    let riya_request =
        client.request_capital(&f.riya, &BID_RIYA, &BytesN::from_array(&f.env, &[10; 32]));
    let aman_request =
        client.request_capital(&f.aman, &BID_AMAN, &BytesN::from_array(&f.env, &[11; 32]));
    let next_round = client.create_round(
        &f.admin,
        &vec![&f.env, riya_request, aman_request],
        &300,
        &400,
    );
    assert_eq!(client.get_round(&next_round).cycle_round_number, 2);
    assert_eq!(client.get_round(&next_round).pot, 50_000 * UNIT);
    assert_eq!(client.get_pool_balance().reserved_pool, 50_000 * UNIT);
    assert_eq!(client.get_pool_balance().discount_liability, 7_000 * UNIT);
    assert_eq!(client.get_pool_balance().token_balance, 57_000 * UNIT);
    let secret = f.secret(140);
    let hash = commitment::hash(&f.env, next_round, &f.riya, BID_RIYA, &secret);
    client.commit_bid(&next_round, &f.riya, &hash);
    assert_eq!(
        client.try_commit_bid(&next_round, &f.rahul, &hash),
        Err(Ok(Error::NotParticipant))
    );
}

#[test]
fn discount_remainder_uses_ascending_wallet_order_and_claim_is_once() {
    let f = AuctionFixture::new();
    let bid = BID_RAHUL - 3;
    let secret = f.secret(141);
    f.commit(&f.rahul, bid, &secret);
    f.open_reveal();
    f.client().reveal_bid(&f.round_id, &f.rahul, &bid, &secret);
    f.env.ledger().set_timestamp(200);
    f.client().finalize_round(&f.round_id);
    f.client().settle(&f.round_id, &f.rahul);
    let round = f.client().get_round(&f.round_id);
    assert_eq!(round.auction_discount, 7_000 * UNIT + 3);
    assert_eq!(round.discount_remainder, 3);
    let mut total = 0;
    for address in f.members.iter() {
        let rank = f
            .members
            .iter()
            .filter(|other| other.to_string() < address.to_string())
            .count();
        let expected = 700 * UNIT + if rank < 3 { 1 } else { 0 };
        let credit = f
            .client()
            .get_cycle_member(&f.cycle_id, &address)
            .claimable_discount;
        assert_eq!(credit, expected);
        total += credit;
    }
    assert_eq!(bid + total, round.pot);
    let claimant = f.members.get(0).unwrap();
    let balance_before = f.client().get_pool_balance().token_balance;
    let amount = f
        .client()
        .get_cycle_member(&f.cycle_id, &claimant)
        .claimable_discount;
    f.env.set_auths(&[]);
    assert!(f
        .client()
        .try_claim_discount(&f.cycle_id, &claimant)
        .is_err());
    f.env.mock_all_auths();
    assert_eq!(f.client().claim_discount(&f.cycle_id, &claimant), amount);
    assert_eq!(
        f.client().get_pool_balance().token_balance,
        balance_before - amount
    );
    assert_eq!(
        f.client().get_pool_balance().discount_liability,
        total - amount
    );
    assert_eq!(
        f.client().try_claim_discount(&f.cycle_id, &claimant),
        Err(Ok(Error::NoDiscountToClaim))
    );
    for address in f.members.iter() {
        if address != claimant {
            f.client().claim_discount(&f.cycle_id, &address);
        }
    }
    let balance = f.client().get_pool_balance();
    assert_eq!(balance.token_balance, 0);
    assert_eq!(balance.discount_liability, 0);
}

#[test]
fn invalid_winner_and_missing_reserved_tokens_block_settlement() {
    let f = AuctionFixture::finalized_with_rahul_winning();
    let client = f.client();
    assert_eq!(
        client.try_settle(&f.round_id, &f.riya),
        Err(Ok(Error::NoWinner))
    );
    let asset = client.get_community().asset;
    let token = soroban_sdk::token::TokenClient::new(&f.env, &asset);
    token.transfer(
        &f.contract,
        &MuxedAddress::from(&f.outsider),
        &(8_000 * UNIT),
    );
    assert_eq!(
        client.try_settle(&f.round_id, &f.rahul),
        Err(Ok(Error::InsufficientPoolBalance))
    );
    assert_eq!(client.get_round(&f.round_id).status, RoundStatus::Finalized);
    assert_eq!(client.get_community().reserved_pool, 50_000 * UNIT);
}

#[test]
fn no_reveal_releases_pot_for_a_retry_of_the_same_cycle_round() {
    let f = AuctionFixture::new();
    f.open_reveal();
    f.env.ledger().set_timestamp(200);
    f.client().finalize_round(&f.round_id);
    assert_eq!(f.client().get_pool_balance().available_pool, 50_000 * UNIT);
    assert_eq!(f.client().get_pool_balance().reserved_pool, 0);
    assert_eq!(f.client().get_cycle(&f.cycle_id).current_round_number, 1);
    assert_eq!(f.client().get_cycle(&f.cycle_id).active_round_id, None);
    assert_eq!(f.client().get_request(&1).status, RequestStatus::Pending);
    let retry = f
        .client()
        .create_round(&f.admin, &vec![&f.env, 1, 2, 3], &300, &400);
    assert_eq!(f.client().get_round(&retry).pot, 50_000 * UNIT);
    assert_eq!(f.client().get_pool_balance().reserved_pool, 50_000 * UNIT);
}

#[test]
fn ten_distinct_payouts_complete_cycle_and_contribution_obligations() {
    let f = AuctionFixture::finalized_with_rahul_winning();
    f.client().settle(&f.round_id, &f.rahul);
    for index in 1..10u32 {
        let recipient = f.members.get(index).unwrap();
        for address in f.members.iter() {
            f.client().contribute(&address);
        }
        let request = f.client().request_capital(
            &recipient,
            &BID_RAHUL,
            &BytesN::from_array(&f.env, &[index as u8; 32]),
        );
        let commit_deadline = 200 + u64::from(index) * 100;
        let reveal_deadline = commit_deadline + 50;
        let round_id = f.client().create_round(
            &f.admin,
            &vec![&f.env, request],
            &commit_deadline,
            &reveal_deadline,
        );
        let secret = f.secret(index as u8);
        let hash = commitment::hash(&f.env, round_id, &recipient, BID_RAHUL, &secret);
        f.client().commit_bid(&round_id, &recipient, &hash);
        f.env.ledger().set_timestamp(commit_deadline);
        f.client().start_reveal(&round_id);
        f.client()
            .reveal_bid(&round_id, &recipient, &BID_RAHUL, &secret);
        f.env.ledger().set_timestamp(reveal_deadline);
        f.client().finalize_round(&round_id);
        f.client().settle(&round_id, &recipient);
    }
    let cycle = f.client().get_cycle(&f.cycle_id);
    assert!(cycle.complete);
    assert_eq!(cycle.payouts_completed, 10);
    for address in f.members.iter() {
        let state = f.client().get_cycle_member(&f.cycle_id, &address);
        assert!(state.payout_received);
        assert!(state.obligation_complete);
        assert_eq!(state.expected_contributions, 10);
        assert_eq!(state.completed_contributions, 10);
        assert_eq!(
            state.contributions_before_payout + state.contributions_after_payout,
            10
        );
        assert_eq!(f.client().get_member(&address).payouts_received, 1);
    }
    assert_eq!(
        f.client().try_contribute(&f.rahul),
        Err(Ok(Error::CycleComplete))
    );
}
