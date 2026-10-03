import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { computeCommitment, encodeCommitment } from "./commitment.ts";

const vectors = readFileSync(new URL("../../../../packages/shared/test-vectors/commitment-v1.txt", import.meta.url), "utf8")
  .split("\n")
  .filter((line) => line && !line.startsWith("#"))
  .map((line) => {
    const [name, roundId, participant, bidAmount, secret, expected] = line.split("|");
    return { name, roundId: BigInt(roundId), participant, bidAmount: BigInt(bidAmount), secret: Buffer.from(secret, "hex"), expected };
  });

test("TypeScript matches every shared Rust commitment vector", async () => {
  assert.equal(vectors.length, 5);
  const actual = await Promise.all(vectors.map(async (vector) =>
    Buffer.from(await computeCommitment(vector.roundId, vector.participant, vector.bidAmount, vector.secret)).toString("hex"),
  ));
  vectors.forEach((vector, index) => assert.equal(actual[index], vector.expected, vector.name));
  actual.slice(1).forEach((hash) => assert.notEqual(hash, actual[0]));
});

test("rejects malformed commitment fields", () => {
  const vector = vectors[0];
  assert.throws(() => encodeCommitment(-1n, vector.participant, vector.bidAmount, vector.secret));
  assert.throws(() => encodeCommitment(vector.roundId, vector.participant, 0n, vector.secret));
  assert.throws(() => encodeCommitment(vector.roundId, vector.participant, vector.bidAmount, new Uint8Array(31)));
});
