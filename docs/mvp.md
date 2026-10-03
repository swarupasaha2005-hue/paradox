# MVP implementation boundary

This commit initializes tooling and placeholder routes only.

Next implementation order:
1. Community/member state and authenticated token contributions.
2. Transparent eligibility and capital requests.
3. Commit/reveal with canonical Soroban encoding binding round, wallet, amount, and secret.
4. Lowest valid bid allocation, deterministic ties, and one-time settlement.
5. Repayment, explicit default transition, and financial history.
6. Testnet deployment and Freighter transaction integration.
7. End-to-end demo and frontend polish.

Acceptance demo: three eligible wallets, 50,000-unit pool, sealed bids of 43,000 / 46,000 / 45,000. Reject 42,000 with Rahul's original secret; accept 43,000. Rahul wins, receives funding once, repays, and updates history. Names and business profiles stay off-chain.

Before implementing the UI, test canonical Rust/TypeScript commitment encoding parity. Bids and secrets must stay local until reveal. Never expose a pre-reveal bid through a contract call or event.

No credit scores, governance, protocol token, KYC, database, indexer, or lending infrastructure. Financial history controls eligibility, never the auction winner.
