# Arth MVP boundary

The demo uses one Soroban community contract and a fixed member cycle. Members make real fixed token contributions each round; the current round's contributions form a reserved pot. Eligible members submit sealed bids for the discounted payout they would accept in exchange for receiving their turn. The lowest valid revealed bid wins, with deterministic ties.

Acceptance demo: 10 members each contribute 5,000 units, making a 50,000-unit round pot. Rahul, Riya, and Aman reveal 43,000, 46,000, and 45,000 respectively. Rahul receives 43,000 once. The 7,000 difference becomes equal claimable discount credits of 700 for all 10 members, including Rahul. Rahul has no separate 43,000 loan debt and continues contributing 5,000 in each remaining cycle round. He cannot receive another payout in this cycle.

Commitments use the fixed Step 1 encoding and are hash-only until reveal. The demo should reject Rahul's attempted 42,000 reveal with his original 43,000 secret. Financial history affects eligibility, never bid ranking. Wallet addresses are protocol identities; business profiles remain frontend metadata.

No credit scores, interest, conventional loan repayment, KYC, governance, protocol token, database, backend, or indexer. The MVP records expected and completed contributions without an automatic overdue/default schedule. See [cycle and auction economics](step3-auction.md).
