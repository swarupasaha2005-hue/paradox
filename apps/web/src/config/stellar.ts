import { STELLAR_TESTNET } from "@paradox/shared";

export const STELLAR_CONFIG = {
  rpcUrl: process.env.NEXT_PUBLIC_STELLAR_RPC_URL || STELLAR_TESTNET.rpcUrl,
  horizonUrl: process.env.NEXT_PUBLIC_STELLAR_HORIZON_URL || STELLAR_TESTNET.horizonUrl,
  networkPassphrase:
    process.env.NEXT_PUBLIC_STELLAR_NETWORK_PASSPHRASE || STELLAR_TESTNET.networkPassphrase,
  contractId: process.env.NEXT_PUBLIC_COMMUNITY_POOL_CONTRACT_ID || "",
  assetContractId: process.env.NEXT_PUBLIC_POOL_ASSET_CONTRACT_ID || "",
} as const;
