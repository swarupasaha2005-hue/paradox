import assert from "node:assert/strict";
import test from "node:test";
import { parseXlm, formatXlm } from "./amount.ts";
import { computeCommitment } from "./commitment.ts";
import { clearRevealedSecret, locallyMatchesCommitment, restoreBid, saveBid, sealedBidKey, toHex, type StoredBid } from "./sealed-bid.ts";

const wallet = "GCGQGZ52W5IYLPBWSX3CHAXT75D4YJFTSQ4JXNLZ6RT7AFJ4IYJKKNWJ";
const secret = Uint8Array.from({ length: 32 }, (_, index) => index + 1);
const memory = () => {
  const values = new Map<string, string>();
  return { getItem: (key: string) => values.get(key) ?? null, setItem: (key: string, value: string) => { values.set(key, value); } };
};

test("display XLM converts to exact integer base units and back", () => {
  assert.equal(parseXlm("5,000", 7), 50_000_000_000n);
  assert.equal(parseXlm("43000.1234567", 7), 430_001_234_567n);
  assert.equal(formatXlm(parseXlm("43000.1234567", 7), 7), "43,000.1234567 XLM");
  assert.throws(() => parseXlm("1.12345678", 7));
  assert.throws(() => parseXlm("-1", 7));
  assert.equal(parseXlm("8,600.123456", 6), 8_600_123_456n);
  assert.equal(formatXlm(8_600_123_456n, 6), "8,600.123456 XLM");
  assert.throws(() => parseXlm("1.1234567", 6));
});

test("stored bid is isolated by contract, wallet and round", async () => {
  const storage = memory();
  const commitment = toHex(await computeCommitment(7n, wallet, 430_000_000_000n, secret));
  const record: StoredBid = { contractId: "CONTRACT_A", wallet, cycleId: "1", roundId: "7", bidBaseUnits: "430000000000", secretHex: toHex(secret), commitmentHex: commitment, createdAt: "2026-10-03T00:00:00Z", revealed: false };
  saveBid(storage, record);
  assert.deepEqual(restoreBid(storage, "CONTRACT_A", wallet, 7n), record);
  assert.equal(restoreBid(storage, "CONTRACT_B", wallet, 7n), null);
  assert.equal(restoreBid(storage, "CONTRACT_A", "GDIFFERENT", 7n), null);
  assert.equal(restoreBid(storage, "CONTRACT_A", wallet, 8n), null);
  assert.notEqual(sealedBidKey("CONTRACT_A", wallet, 7n), sealedBidKey("CONTRACT_A", wallet, 8n));
  assert.equal(await locallyMatchesCommitment(record, commitment), true);
  assert.equal(await locallyMatchesCommitment(record, "00".repeat(32)), false);
  assert.equal(await locallyMatchesCommitment({ ...record, bidBaseUnits: "420000000000" }, commitment), false);
  const cleared = clearRevealedSecret(storage, record);
  assert.equal(cleared.secretHex, "");
  assert.deepEqual(restoreBid(storage, "CONTRACT_A", wallet, 7n), cleared);
  assert.equal(await locallyMatchesCommitment(cleared, commitment), false);
});
