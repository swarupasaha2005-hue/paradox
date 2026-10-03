"use client";

import { useState } from "react";
import { getNetworkDetails, requestAccess, signTransaction } from "@stellar/freighter-api";
import { BASE_FEE, Contract, TransactionBuilder, rpc } from "@stellar/stellar-sdk";
import { STELLAR_CONFIG } from "@/config/stellar";

export default function IntegrationSpikePage() {
  const [contractId, setContractId] = useState(STELLAR_CONFIG.contractId);
  const [status, setStatus] = useState("Ready");
  const [hash, setHash] = useState("");
  const [busy, setBusy] = useState(false);

  async function invokeVersion() {
    setBusy(true);
    setHash("");
    try {
      const network = await getNetworkDetails();
      if (network.error) throw new Error("Could not read Freighter network");
      if (network.networkPassphrase !== STELLAR_CONFIG.networkPassphrase) {
        throw new Error("Select Stellar Testnet in Freighter before signing.");
      }
      const access = await requestAccess();
      if (access.error || !access.address) throw new Error("Freighter account access was denied.");
      const server = new rpc.Server(STELLAR_CONFIG.rpcUrl);
      setStatus("Loading account and preparing contract invocation…");
      const account = await server.getAccount(access.address);
      const tx = new TransactionBuilder(account, {
        fee: BASE_FEE,
        networkPassphrase: STELLAR_CONFIG.networkPassphrase,
      }).addOperation(new Contract(contractId.trim()).call("version")).setTimeout(60).build();
      const prepared = await server.prepareTransaction(tx);
      setStatus("Waiting for Freighter approval…");
      const signed = await signTransaction(prepared.toXDR(), {
        networkPassphrase: STELLAR_CONFIG.networkPassphrase,
        address: access.address,
      });
      if (signed.error || !signed.signedTxXdr) throw new Error("Freighter signing was denied.");
      setStatus("Submitting signed transaction to Testnet…");
      const signedTx = TransactionBuilder.fromXDR(signed.signedTxXdr, STELLAR_CONFIG.networkPassphrase);
      const submitted = await server.sendTransaction(signedTx);
      if (submitted.status === "ERROR") throw new Error("RPC rejected the signed transaction.");
      setHash(submitted.hash);
      for (let attempt = 0; attempt < 20; attempt++) {
        const result = await server.getTransaction(submitted.hash);
        if (result.status === "SUCCESS") {
          setStatus("Confirmed: signed version() invocation succeeded on Testnet.");
          return;
        }
        if (result.status === "FAILED") throw new Error(`Transaction failed: ${submitted.hash}`);
        await new Promise((resolve) => setTimeout(resolve, 1500));
      }
      throw new Error("Confirmation timed out; check the transaction hash.");
    } catch (error) {
      setStatus(error instanceof Error ? error.message : "Invocation failed.");
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="mx-auto max-w-xl space-y-5 p-8">
      <h1 className="text-2xl font-semibold">Testnet integration spike</h1>
      <p>Invoke the contract&apos;s read-only <code>version()</code> method in a signed Testnet transaction. The wallet pays a network fee.</p>
      <label className="block space-y-2">
        <span>Testnet contract ID</span>
        <input className="w-full rounded border p-2" value={contractId} onChange={(event) => setContractId(event.target.value)} placeholder="C…" />
      </label>
      <button className="rounded bg-slate-900 px-4 py-2 text-white disabled:opacity-50" disabled={busy || !contractId.trim()} onClick={invokeVersion}>
        Sign and invoke with Freighter
      </button>
      <p role="status">{status}</p>
      {hash && <p className="break-all">Transaction hash: {hash}</p>}
    </main>
  );
}
