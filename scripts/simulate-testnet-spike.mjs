import { BASE_FEE, Contract, Networks, TransactionBuilder, rpc, scValToNative } from "@stellar/stellar-sdk";

const [contractId, accountAddress] = process.argv.slice(2);
if (!contractId || !accountAddress) {
  throw new Error("Usage: node scripts/simulate-testnet-spike.mjs <testnet-contract-id> <funded-public-address>");
}

const server = new rpc.Server("https://soroban-testnet.stellar.org");
const account = await server.getAccount(accountAddress);
const transaction = new TransactionBuilder(account, {
  fee: BASE_FEE,
  networkPassphrase: Networks.TESTNET,
}).addOperation(new Contract(contractId).call("version")).setTimeout(60).build();
const prepared = await server.prepareTransaction(transaction);
const simulation = await server.simulateTransaction(transaction);
if (rpc.Api.isSimulationError(simulation)) throw new Error(`RPC simulation failed: ${simulation.error}`);
if (!simulation.result || scValToNative(simulation.result.retval) !== 1) throw new Error("Unexpected version() result");
console.log(`Prepared signed invocation XDR: ${prepared.toXDR().length} base64 characters`);
console.log("RPC simulation: version() returned 1");
