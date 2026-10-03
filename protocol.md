# Protocol Integration PRD — Community Finance Protocol

**Goal:** Deliver one working Stellar Testnet demo of a rotating community-finance cycle: members make fixed contributions each round, eligible members commit and reveal sealed bids for that round's discounted pot payout, the lowest valid bid wins, and the auction discount becomes claimable credits for cycle members. A winning bid is a payout amount, not loan principal.

## Repository assessment

The repository uses Next.js, TypeScript, npm workspaces, Stellar SDK, Freighter API, and one Rust/Soroban contract. The six frontend routes remain placeholders. The contract implements cycles, token contributions, auction rounds, payouts, and discount credits; final Testnet deployment and frontend integration are later stages. The visible product name is centralized in `apps/web/src/config/product.ts`; the contract is in `contracts/community_pool/src/lib.rs`.

## MVP requirements

| Area | Required behavior | Acceptance check |
| --- | --- | --- |
| Community and cycle | Create one community, join with wallets, then fix up to 10 active members in a cycle | Each member has a cycle contribution and payout record |
| Pool | Transfer the configured asset into the contract once per cycle member per round | Ten contributions of 5,000 make the current round pot 50,000 |
| Eligibility | Require current-cycle membership, current-round contribution, no prior payout in this cycle, no unresolved default, and a request within the limit | UI shows the exact pass or fail reason |
| Requests | Eligible members submit a maximum discounted payout amount for the current round | An eligible request can enter the current round |
| Commit | Store one commitment per wallet per round; never publish bid or secret before reveal | On-chain commitment reveals neither value |
| Reveal | Recompute the commitment from round ID, wallet, bid, and secret | Rahul's `42,000` reveal with his original secret is rejected; `43,000` succeeds |
| Allocation | After reveal closes, select the lowest valid bid; use a fixed wallet-address order for ties | Rahul wins against bids of 45,000 and 46,000 |
| Settlement | Transfer the winning bid once from the reserved pot to the winner | Rahul receives 43,000 and cannot win again in this cycle |
| Auction discount | Credit the 7,000 difference equally to all 10 cycle members, including Rahul | Each member can claim 700 exactly once; there is no separate loan repayment |

Financial history determines **eligibility only**. It must never rank auction bids. Seeded names and businesses are frontend metadata; wallet addresses are the protocol identities.

**Verifiable participation history:** Public reads expose objective wallet facts from actual contributions, payouts, discount credits and claims, and completed cycles. A member's aggregate history persists across cycles, and prior cycle records remain readable by cycle ID. These facts are not a traditional credit score, guarantee of creditworthiness, AI-generated risk rating, or promise of future payment. Another community or frontend can inspect them from the relevant CommunityPool contract ID; there is no cross-contract registry in the MVP. Without a defined calendar deadline, an unpaid contribution is not labeled as missed or defaulted.

## Protocol design

Use the existing single `CommunityPool` Soroban contract and a configured [Stellar token-interface compatible asset](https://developers.stellar.org/docs/tokens/token-interface). Keep amounts as integer token units on-chain; convert to display units only in the UI. No backend or database is required.

**Core state:** Community configuration and admin; fixed cycle membership and payout status; expected and completed contributions before and after payout; available, reserved, and discount-liability accounting; payout requests; round pot, phase, and deadlines; per-wallet commitments and verified reveals; winner and discount credits. Store records needed for correctness in Soroban persistent storage and account for [storage TTL](https://developers.stellar.org/docs/learn/fundamentals/contract-development/storage/state-archival).

**Contract actions:** `create_community`, `join`, `create_cycle`, `contribute`, `request_capital`, `create_round`, `commit_bid`, `start_reveal`, `reveal_bid`, `finalize_round`, `settle`, `claim_discount`, and read methods for community, cycle, member, eligibility, request, round, and pool state. Wallets authenticate actions involving their funds or records.

**Round state:** `Commit → Reveal → Finalized → Settled`. Each cycle round's pot equals its fixed member count times the contribution amount, after every cycle member pays once. Round creation reserves that exact pot; historical discount credits do not enter it, and another active round cannot use it. Ledger-time deadlines close commit and reveal. Unrevealed commitments cannot win. A no-winner round releases its reservation for a retry. A bid must be positive and no greater than its request maximum or the reserved pot. Finalization and settlement are each one-time actions. Settlement pays the bid and credits `pot - bid` across every cycle member, including the winner; integer remainder units go to members in ascending wallet-address order. The winner continues contributing in later rounds and owes no separate repayment of the bid.

**Commitment:** Specify one domain-separated, canonical byte encoding for `(roundId, participant wallet, bidAmount, secret)` and hash it with SHA-256. Implement the same encoding in Rust and TypeScript and prove parity with fixed test vectors before building the UI. Generate a random secret locally and retain it locally until reveal; warn the bidder that losing it prevents reveal. Never place bid or secret in a pre-reveal event, public environment variable, or protocol read response.

## Frontend integration

A small client module should own RPC reads, transaction preparation, Freighter signing, submission, confirmation, and error mapping. This follows Stellar's documented [frontend transaction flow](https://developers.stellar.org/docs/build/guides/dapps/frontend-guide). Use the existing routes:

- `/dashboard`: Wallet, pool, eligibility, active round, history summary.
- `/community`: Create or join, contribute, view members and pool.
- `/request`: Enter amount and see eligibility reasons.
- `/round/[id]`: Phase countdown, commit, reveal, invalid-reveal error, winner, settlement.
- `/history`: On-chain contribution, payout, and discount history.

The landing page can explain the demo and link to the dashboard. Use `NEXT_PUBLIC_*` values only for public network, contract, and asset IDs.

## Demo and verification

Prepare 10 funded Testnet wallets, including Rahul, Riya, and Aman, with frontend metadata for names. Have each contribute 5,000 token display units into one cycle round. Run a 50,000-unit pot auction with valid bids of 43,000, 46,000, and 45,000. During reveal, submit Rahul's incorrect 42,000 amount with his original secret and show the contract rejection; then submit 43,000, finalize, and settle. Verify the 43,000 payout and ten 700-unit discount credits. Rahul remains obligated to contribute in the next round and cannot receive another payout in the same cycle.

Required contract tests cover hash mismatch, another wallet's reveal, duplicate or late commit, early or duplicate finalization, invalid winner, duplicate settlement, pot reservation, deterministic discount division, token conservation, one-time claims, and continued contribution after payout. A browser smoke test should complete the same flow on Testnet.

## Six-hour implementation order

1. **0:00–0:35:** Establish Rust/TypeScript commitment encoding parity and test a signed Freighter contract invocation. This is the critical integration spike.
2. **0:35–2:35:** Implement community, authenticated contributions, eligibility, requests, and round state.
3. **2:35–3:35:** Implement commit, reveal, finalization, cycle payout settlement, discount accounting, and focused contract tests.
4. **3:35–4:15:** Deploy to Testnet, configure asset and contract IDs, fund demo wallets.
5. **4:15–5:30:** Connect the existing routes to contract reads and writes; add clear transaction and rejection states.
6. **5:30–6:00:** Run the full three-wallet demo, fix blockers, and rehearse the invalid reveal.

## Explicit MVP cuts

No conventional loan repayment, AI credit score, interest model, governance, tokenomics, NFT, KYC, database, indexer, business documents, or production underwriting. Expected and completed contributions are recorded. Do not increment missed/default counters without a defined schedule and due-date rule; an explicit overdue-default transition can be added only after the main demo works.

**READY TO IMPLEMENT: YES.** The highest-risk dependency is identical commitment encoding and hashing across the browser and Soroban contract, followed immediately by a real Freighter-signed Testnet call.
