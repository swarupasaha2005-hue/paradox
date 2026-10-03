# Community Finance Protocol

Internal repository name: `paradox`. A six-hour hackathon MVP for community working-capital pools on Stellar Testnet. Transparent participation history determines eligibility; a sealed-bid commit/reveal auction allocates scarce capital to the lowest valid revealed bid.

## Architecture

- `apps/web`: Next.js App Router, TypeScript, Tailwind CSS, and Lucide. Stellar SDK and Freighter API are installed for later integration.
- `contracts/community_pool`: one Rust/Soroban `CommunityPool` contract for the eventual pool, requests, auction, settlement, and repayment state.
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

Open http://localhost:3000. The contract and asset IDs can remain empty for placeholder pages. Public environment variables must never contain wallet private keys or bid secrets.

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

For a production frontend process locally: `npm run start --workspace @paradox/web` after building. Deployment is deferred: deploy the implemented contract to Stellar Testnet, then set its contract ID and asset contract ID before building the frontend. No contract has been deployed by this initialization.

## Current implementation status

- Frontend foundation and placeholder routes: `/`, `/dashboard`, `/community`, `/request`, `/round/[id]`, `/history`.
- Contract scaffold exposes only `version() -> 1`; no financial behavior is implemented.
- Testnet configuration and wallet dependencies are present; wallet connection, signing, and RPC calls are not implemented.
- Commit/reveal, eligibility, contributions, settlement, repayment, history, and demo seeding await implementation.
- No protocol tests yet. Initialization is checked by lint, TypeScript, frontend production build, Rust formatting/checking, and the Stellar WASM build.
- shadcn/ui components and animation dependencies will be added only when needed during frontend implementation.

Verification: the combined check script and HTTP smoke checks for all six routes passed locally. `npm audit --omit=dev` reports zero vulnerabilities. The full audit reports five high findings in the development-only ESLint dependency chain (`braces` / `micromatch` / `fast-glob`); no forced framework downgrade was applied.
