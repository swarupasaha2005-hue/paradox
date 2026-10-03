# Community Finance Protocol

Internal repository name: `paradox`. Arth is a rotating community-finance MVP on Stellar Testnet. Members contribute a fixed amount each cycle round; a sealed-bid auction chooses who receives that round's discounted pot payout.

## Architecture

- `apps/web`: Next.js App Router, TypeScript, Tailwind CSS, and Lucide. Stellar SDK and Freighter API are installed for later integration.
- `contracts/community_pool`: one Rust/Soroban `CommunityPool` contract for cycles, contributions, reserved round pots, sealed bids, payouts, and discount credits.
- `packages/shared`: shared Stellar Testnet constants; future shared protocol types.
- `scripts/check.sh`: repeatable scaffold checks.
- `docs/mvp.md`: implementation boundary and acceptance demo.

Simple npm and Cargo workspaces; no backend, database, or Turborepo. Business names and descriptions stay off-chain. Wallets are the on-chain identities.

The visible product name and description are centralized in `apps/web/src/config/product.ts`, including page metadata. Rename them there when the final identity is chosen.

## Local development

Prerequisites: Node.js 22.12+ (verified locally with Node 24), npm, Rust 1.91+, and Stellar CLI 27. The scaffold pins Soroban SDK 27.0.0 and commits both dependency lockfiles.

```sh
npm ci
rustup target add wasm32v1-none
cp apps/web/.env.example apps/web/.env.local
npm run dev
```

Open http://localhost:3000. The example environment points to the initialized [Testnet CommunityPool](docs/testnet-deployment.md). Public environment variables must never contain wallet private keys or bid secrets.

```sh
npm run lint
npm run typecheck
npm run build
cargo fmt --all -- --check
cargo check --workspace --locked
npm run contract:build
# Or run all the above:
npm run check
```

Built contract: `target/wasm32v1-none/release/community_pool.wasm`.

For a production frontend process locally: `npm run start --workspace @paradox/web` after building. The full contract is deployed and initialized on Testnet; frontend wallet and contract integration remains a separate step.

## Current implementation status

- Arth's `/app`, `/app/cycle`, `/app/bid`, and `/app/history` read the deployed Testnet contract and use Freighter for signed member actions. The landing page retains illustrative examples.
- The contract supports one community, authenticated membership, fixed member cycles, real token contributions, structured eligibility, payout requests, and reserved round pots. See [contract state notes](docs/step2-contract.md).
- Rust and TypeScript commitment hashes share five fixed vectors. The contract uses the same Rust hash to verify reveals.
- The Step 1 Testnet `version()` integration spike is obsolete; Freighter signing still awaits wallet setup. See [Step 1 integration notes](docs/step1-integration.md) and the [current Testnet deployment](docs/testnet-deployment.md).
- Sealed-bid commitment and reveal, winner selection, discounted payout, and one-time discount claiming are implemented on-chain. The winning bid is the payout amount, not loan principal. See [cycle and auction notes](docs/step3-auction.md). The live app now builds and signs these calls through Freighter; the complete multi-wallet Testnet lifecycle still awaits participant signatures.
- Public [financial-history reads](docs/step4-history.md) expose objective contribution, payout, discount, and completed-cycle facts by wallet and cycle. They do not assign a credit score. The app displays these reads for connected members.
- Contract tests cover token balances, cycle obligations, reserved pots, adversarial bids, payouts, discount conservation, and claims. Run `cargo test --workspace --locked` in addition to `npm run check` and `npm run test:commitment`.
- shadcn/ui components and animation dependencies will be added only when needed during frontend implementation.

Verification: the combined check script and HTTP smoke checks for all six routes passed locally. `npm audit --omit=dev` reports zero vulnerabilities. The full audit reports five high findings in the development-only ESLint dependency chain (`braces` / `micromatch` / `fast-glob`); no forced framework downgrade was applied.
