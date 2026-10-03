# Step 2 contract state and units

One `CommunityPool` contract represents one community. `create_community` accepts an admin wallet, a deployed SEP-41 token-compatible contract address, an exact contribution amount in **integer token base units**, a minimum contribution count, and a financing limit in the same base units. It requires admin authorization, checks the token's `decimals()` interface, and can run only once.

The community configuration, counters, and accounted available pool are stored together in Soroban instance storage. Member records, capital requests, and rounds use persistent storage keyed by wallet or ID. Mutations extend each affected entry's TTL when it falls below half the network maximum; reads remain read-only. Archived entries will require standard Soroban restoration if the community is inactive beyond their TTL. No indexer or separate archive service is included.

`contribute` calls the configured token's `transfer` from the authenticated member to the contract for exactly the configured amount. It updates accounting only after the transfer returns successfully. Soroban transaction atomicity rolls back both token and protocol changes on failure. `get_pool_balance` exposes both accounted available pool and actual token balance. `create_round` uses the smaller of those balances for its capital snapshot. A request's maximum may exceed the snapshot; Step 3 must constrain each bid by both the request maximum and available capital.

`get_eligibility` returns each rule as a boolean, plus `contributions_completed` and `minimum_contributions`, so the frontend can say “Needs 3 contributions” with the actual progress. `request_capital` and `create_round` call the same eligibility function. Financial counters are not used to rank bids. Names and businesses stay off-chain.

For display, read the configured token's `decimals()`. If it returns `7`, convert `50,000` display units to `50_000 * 10^7` integer token units before contract calls; convert returned integers back only for display. The tests use a test token with 7 decimals, ten contributions of `5_000 * 10^7` each, and verify a `50_000 * 10^7` pool. No floating-point amounts enter contract calls.

Round request lists are capped at 10, with one request per participant in a round. Creating a round changes included requests from `Pending` to `IncludedInRound`, preventing the same request from entering another round. The stored phase is `Commit` with ledger-time commit and reveal deadlines. Bidding, reveal, finalization, settlement, and repayment are Step 3 work.

The installed Soroban SDK 27 test utilities require `ed25519-dalek` 2.x; `Cargo.lock` pins compatible version 2.2.0 because unconstrained resolution selected incompatible 3.0.0 during Step 2 test compilation.
