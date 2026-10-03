function scaleFor(decimals: number): bigint {
  if (!Number.isInteger(decimals) || decimals < 0 || decimals > 18) throw new Error("Invalid asset decimals.");
  return 10n ** BigInt(decimals);
}

export function parseXlm(value: string, decimals: number): bigint {
  const trimmed = value.trim().replaceAll(",", "");
  const pattern = decimals === 0 ? /^(0|[1-9]\d*)$/ : new RegExp(`^(0|[1-9]\\d*)(?:\\.(\\d{1,${decimals}}))?$`);
  const match = trimmed.match(pattern);
  if (!match) throw new Error(`Enter an XLM amount with at most ${decimals} decimal places.`);
  return BigInt(match[1]) * scaleFor(decimals) + BigInt((match[2] ?? "").padEnd(decimals, "0"));
}

export function formatXlm(baseUnits: bigint | string, decimals: number): string {
  const scale = scaleFor(decimals);
  const amount = BigInt(baseUnits);
  const negative = amount < 0n;
  const absolute = negative ? -amount : amount;
  const whole = absolute / scale;
  const fraction = (absolute % scale).toString().padStart(decimals, "0").replace(/0+$/, "");
  return `${negative ? "−" : ""}${whole.toLocaleString("en-US")}${fraction ? `.${fraction}` : ""} XLM`;
}
