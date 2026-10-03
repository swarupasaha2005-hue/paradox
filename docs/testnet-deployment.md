# Arth CommunityPool Testnet deployment

Deployed and initialized on **2026-10-03**. This is the full Steps 1–4 CommunityPool, not the obsolete Step 1 `version()`-only integration spike.

| Item | Value |
| --- | --- |
| Network | Stellar Testnet |
| Network passphrase | `Test SDF Network ; September 2015` |
| RPC | `https://soroban-testnet.stellar.org` |
| Contract ID | `CCFDZNXT3C6K7ZP4Q5MOVMJVJ4QQ5MD4F7M4FXWZWCLA6BHEOIPMZJJR` |
| WASM SHA-256 | `b6cb0d0b1ba5bf45408ab150ec6f4f013f084662dbdef05085d29d8ec5cbe083` |
| Exported contract methods | 26 |
| `version()` | `1` |
| Deployment/admin identity | Existing Stellar CLI `admin`; public address `GCGQGZ52W5IYLPBWSX3CHAXT75D4YJFTSQ4JXNLZ6RT7AFJ4IYJKKNWJ` |
| Asset | Native Testnet XLM through its Stellar Asset Contract |
| Asset contract ID | `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC` |
| Asset decimals | 7 |
| WASM upload | `758c945475a7b3d4a011efb5405686c58f51b00bc07b2011e67c5bf82afdaada` (ledger 4999474) |
| Contract creation | `0462bcd35d7e3bed101bda749fe3a598c978e71bd6d6b0815c6b731931bbc0ed` (ledger 4999476) |
| Community initialization | `ec852e90fd6e1b844ba582078f550e1f9c8a33d59cf7f7b3967375157a6160ca` (ledger 4999484) |

The authenticated `create_community` call succeeded. Its on-chain configuration is:

| Field | On-chain base units | Display units |
| --- | ---: | ---: |
| Fixed contribution | 50,000,000,000 | 5,000 XLM |
| Minimum contributions for payout eligibility | 1 | — |
| Financing limit | 500,000,000,000 | 50,000 XLM |

All contract amounts are integers in XLM base units (1 XLM = 10,000,000 units). A ten-member round with one 5,000 XLM contribution per member has a 50,000 XLM pot. The frontend must convert with integer or bigint arithmetic and must commit the base-unit bid amount. XLM is the demo value unit, not a new Arth token.

Live read simulations after initialization returned `version() = 1`; `get_community()` with `initialized = true`, the IDs and parameters above, `member_count = 0`, and `current_cycle_id = null`; `get_pool_balance()` with all balances zero; `is_member(admin) = false`; and `get_eligibility(admin, 430000000000)` with `eligible = false` and `is_member = false`. The native XLM asset contract returned `decimals() = 7` and a CommunityPool token balance of zero. The deployed WASM hash matches the locally built artifact, and the on-chain interface exports all 26 methods, including `create_community`, `create_cycle`, `contribute`, `commit_bid`, `reveal_bid`, `settle`, and financial-history reads.

## Participant setup still needed

No demo member or cycle was fabricated at deployment time. Each real participant wallet must be funded with Testnet XLM and sign its own `join(member)` call. After up to ten active members have joined, the admin signs `create_cycle(admin, members)` with the desired member addresses. Members then sign each round's fixed `contribute(member)` transfer. The contract requires at least one and at most ten members per cycle. To demonstrate the intended ten-member 50,000 XLM pot, use ten distinct consenting, authenticated participant wallets; do not reuse one wallet as ten members. At 5,000 XLM per member per round, participants will need enough Testnet XLM for each round and fees.

During Step 5B on 2026-10-03, Freighter wallet `GBA4PHFEYML7FYSBS7RN4YUMGCOO6CPRZ57ZHGQ7BWOBRUC3IXUXDKPJ` signed `join`; transaction `6de7dfe76038c325a1cde12967394f29a35775a5fef0d7f50ffc01b85179477b` succeeded in ledger 4999784. Independent Testnet reads confirmed `is_member = true`, an active member record, and `member_count = 1`. No cycle existed after this join.

Another Freighter wallet, `GBF4KEPCUXPP6GIEI4ZO2S4R272STYUMHGLTOCV3HTABEM6GBFOG2XTY`, signed `join` in transaction `5f2cae770777dd3e0588bd789c64afc98f202858c80940664c2148e5663b4c08` (ledger 4999873). The admin then signed `create_cycle` with these two authenticated wallet addresses in transaction `5b5c62d28a7dada9c4707778fa26cad4e50b41d29f52e070e29dcb9f8e50657d`. Live reads showed Cycle #1, Round 1 of 2, both wallets included, zero completed contributions, and a conceptual 10,000 XLM pot once both 5,000 XLM contributions are made. The current round cannot proceed to allocation until both sign their contributions.

The configured admin identity is held by Stellar CLI and has not been imported into Freighter. Browser admin controls appear only if Freighter connects the exact admin address. For this demo, `create_cycle` and `create_round` can remain CLI-operated with the existing `admin` alias; no admin secret goes in the browser. `start_reveal` and `finalize_round` are permissionless in the contract, while `settle` requires the winning wallet's signature. The cycle screen shows the connected wallet's full public address for admin setup; `MemberJoined` events provide an independent record of joined addresses.

Testnet XLM can be requested for each participant through [Stellar Friendbot](https://developers.stellar.org/docs/tools/lab/account) or transferred from another funded Testnet account. Friendbot funding is for the Stellar account; the same native balance is available through the native XLM Stellar Asset Contract's token interface. Friendbot has rate limits and Testnet is periodically reset. No issuer, custom asset, or trustline is needed for native XLM.

## Public frontend configuration

`apps/web/.env.example` records the public Testnet RPC, passphrase, CommunityPool ID, and asset contract ID. Copy it to `apps/web/.env.local` for local development. No secret keys or bid secrets belong in either file. The Step 5B app uses these values for live reads and Freighter-signed user transactions.
