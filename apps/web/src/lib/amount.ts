export const XLM_DECIMALS = 7;
export const XLM_SCALE = 10n ** BigInt(XLM_DECIMALS);

export function parseXlm(value: string): bigint {
  const trimmed = value.trim().replaceAll(",", "");
  if (!/^(0|[1-9]\d*)(?:\.\d{1,7})?$/.test(trimmed)) throw new Error("Enter an XLM amount with at most 7 decimal places.");
  const [whole, fraction = ""] = trimmed.split(".");
  return BigInt(whole) * XLM_SCALE + BigInt(fraction.padEnd(XLM_DECIMALS, "0"));
}

export function formatXlm(baseUnits: bigint | string, decimals = XLM_DECIMALS): string {
  const amount = BigInt(baseUnits);
  const negative = amount < 0n;
  const absolute = negative ? -amount : amount;
  const whole = absolute / XLM_SCALE;
  const fraction = (absolute % XLM_SCALE).toString().padStart(XLM_DECIMALS, "0");
  const shown = fraction.slice(0, decimals).replace(/0+$/, "");
  return `${negative ? "−" : ""}${whole.toLocaleString("en-US")}${shown ? `.${shown}` : ""} XLM`;
}

export function formatXlmExact(baseUnits: bigint | string): string {
  return formatXlm(baseUnits, XLM_DECIMALS);
}
