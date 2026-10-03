# Step 1: commitment encoding and Testnet spike

## Commitment V1

`SHA-256(domain || round || address_length || address || amount || secret)` with no separators or text conversion:

| Field | Encoding |
| --- | --- |
| domain | 24 ASCII bytes: `COMMUNITY_FINANCE_BID_V1` |
| round | unsigned 64-bit integer, big-endian, 8 bytes |
| address_length | unsigned 32-bit integer, big-endian, length of the next field |
| address | canonical XDR of `ScVal::Address` for a Stellar `G...` account wallet (44 bytes for this address type) |
| amount | positive signed 128-bit token-unit integer, big-endian two's complement, 16 bytes |
| secret | exactly 32 random bytes |

The address XDR includes the Soroban value and address type discriminants. The total V1 preimage for a `G...` wallet is 128 bytes. Hash computation stays internal to the contract; invoking a public hash method before reveal would publish the bid and secret in a transaction. The five shared vectors under `packages/shared/test-vectors/` are **public test data only**. Real secrets must be generated locally with a cryptographically secure RNG, kept locally until reveal, and never logged or placed in public configuration. Stage 3 will call the same Rust function from reveal verification.

Run `cargo test --workspace --locked` and `npm run test:commitment` to check both implementations against the same vectors. The base vector and four single-field mutations must all differ.

## Testnet spike

The Step 1 `version()`-only contract was deployed solely to exercise the integration path. **It is obsolete and must not be used for Arth protocol calls.** The full CommunityPool deployment is recorded in [Testnet deployment](testnet-deployment.md).

- Contract ID: `CCMNYWLCQBDU4FYOXXQXY3EL6UFDGZMHC4XR4JVDQ3ZVBLK6KRQJW6O4`
- WASM hash: `444318947963ffbdb20592628bd0e3eba38929175d740ecff686573fcb123d6b`
- WASM upload transaction: `1d978e19f93deeebe6afcf66bb78bf35ac513b577d42767be9dd58467356adb2`
- Contract creation transaction: `00b0fe9cc59a390ca844d84506ef654274cb384820e61e04268ba4b6836db1dc`
- Temporary, funded public account for RPC simulation: `GBIARMJUGXJ7SRM4KMFYHT4A2XIPNWRHCH6VH453LXBKKSFGAHLHZTZE`

The CLI read and SDK RPC simulation both returned `version() == 1`. The SDK also prepared an invocation envelope. The browser path at `/integration-spike` checks Freighter's Testnet selection, obtains wallet access, prepares the invocation via RPC, requests Freighter signing, submits, and waits for confirmation. It has **not** produced a signed Freighter invocation: the installed Freighter extension opened at first-run wallet setup. Creating its password and approving the signature require the wallet owner. No private key or wallet setup data belongs in this repository.
