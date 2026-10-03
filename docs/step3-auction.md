# Rotating community-finance cycle

A bid is the discounted amount a member is willing to receive from the **current community pot** in exchange for receiving their turn in that round. It is **not loan principal**. One cycle has a fixed list of 1–10 active members and at most one payout per member. The admin calls `create_cycle` after members join. A cycle starts at contribution round 1, advances after each settled payout, and completes when every cycle member has received one payout. The same community can start another cycle after completion.

Each cycle member owes exactly one configured contribution in each cycle round, including rounds after their payout. `contribute` authenticates the member, transfers that amount to the contract, and records the current round as paid. A second contribution in the same round fails. `CycleMember` exposes expected and completed contributions, counts made before and after payout, payout status, and whether the cycle obligation is complete. There is no calendar-based missed-payment or default transition in this MVP. The configured minimum contribution history applies up to the current round number; a member must also have paid the current round before requesting or bidding. A prior payout in the cycle makes the member ineligible for another payout.

## Pot and reservation

All cycle members must contribute for the current round before the admin can create its auction. The pot is **cycle member count × fixed contribution amount**. `create_round` checks that both accounted available funds and actual token balance cover the pot, moves the pot from `available_pool` to `reserved_pool`, and records it on the round. Only one round may be active for a cycle. Historical discount credits remain in `discount_liability` and are never counted in a later pot. The contract checks that actual token balance covers `available_pool + reserved_pool + discount_liability` before reservation, settlement, and discount claims. A finalized round with no valid reveal releases its reservation and returns its requests to `Pending`, allowing a fresh auction for the same contribution round.

## Sealed-bid auction

`commit_bid(round_id, participant, commitment)` requires participant authorization, an included request, current-cycle eligibility, `Commit` status, and ledger time before the commit deadline. It stores one hash per participant and round, with no bid or secret. The unchanged [Step 1 canonical hash](step1-integration.md) is recomputed by `reveal_bid` after `start_reveal`. Revealed bids must be positive, no greater than the participant's request maximum or the reserved pot, and submitted before the reveal deadline. Mismatches and duplicate reveals fail. Accepted amounts are public; secrets are never stored. Real bidders must generate and retain a cryptographically random 32-byte secret locally.

After the reveal deadline, `finalize_round` chooses the lowest valid revealed bid, with equal bids resolved by ascending wallet StrKey. Unrevealed commitments cannot win. Finalization is one-time. The round phases are `Commit → Reveal → Finalized → Settled`.

## Payout and auction discount

The authenticated winner calls `settle(round_id, winner)` once. If the pot is 50,000 and the winning bid is 43,000, the contract transfers **43,000** to the winner and computes a **7,000 auction discount**. No separate 43,000 debt or loan repayment exists. The winner's request becomes `PaidOut`; other included requests become `NotSelected`. The winner cannot receive a second payout in the cycle, but must keep making the same fixed contributions in later rounds.

Every cycle member, **including the winner**, receives an equal internal discount credit. For 10 members and a 7,000 discount, each receives 700. Division uses integer base units. If the discount is not divisible by the member count, the floor share goes to everyone and one extra base unit goes to each of the first `remainder` members in ascending wallet StrKey order. Thus the full discount is credited deterministically; no unallocated token remains. `claim_discount(cycle_id, member)` authenticates the member, transfers their accumulated credit once, clears it, and reduces `discount_liability`. Credits can accumulate across rounds until claimed.

For each settled round, the checked accounting identity is:

`winner payout + (equal share × cycle members) + distributed remainder = round pot`.

The corresponding token liabilities are `available_pool + reserved_pool + discount_liability`; direct unsolicited token transfers may make actual contract balance larger, but must never make it smaller than these accounted amounts. All amounts are integer token base units. The frontend must use the configured token's decimals for display conversion.

Objective wallet participation facts and per-cycle progress are exposed through the [financial-history reads](step4-history.md).
