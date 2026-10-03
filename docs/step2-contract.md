# Community contract state and units

One `CommunityPool` contract represents one community. `create_community` accepts an admin wallet, a SEP-41 token-compatible asset address, the fixed contribution amount in integer token base units, a minimum contribution count, and a financing limit on individual payout requests. It requires admin authorization and can run only once. Members join with wallet authentication; personal names and documents stay off-chain.

A subsequent `create_cycle` fixes 1–10 active members for that cycle. Each member contributes once per cycle round. `contribute` performs a real token transfer and updates history only after success. `get_eligibility` reports membership, active status, current-cycle participation and contribution, minimum contribution progress, prior payout status, default flag, and request amount validity. Requests and round creation use the same eligibility rules. Financial history never ranks bids.

The community configuration and aggregate accounting use Soroban instance storage. Member, cycle, cycle-member, request, round, commitment, and reveal records use persistent storage, with TTL extension on writes. Standard Soroban restoration is needed if long-inactive records archive. No indexer is included.

The current round pot is the fixed contribution amount multiplied by the cycle member count, after all members pay. Round creation reserves this pot; historical discount liabilities are separate. See [cycle and auction economics](step3-auction.md). For display, read the asset's `decimals()`: with seven decimals, 50,000 display units become `50_000 * 10^7` integer base units. No floating-point amounts enter the contract.
