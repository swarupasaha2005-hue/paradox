"use client";
import { useArth } from "@/lib/arth-context";

export function LiveState({ children }: { children: React.ReactNode }) {
  const { loadState, readError, snapshot, refresh } = useArth();
  if (loadState === "loading" && !snapshot) return <div className="app-live-state" role="status">READING STELLAR TESTNET…</div>;
  if (loadState === "error" || !snapshot) return <div className="app-live-state" role="alert"><strong>CONTRACT READ FAILED</strong><p>{readError || "Unable to load the current contract state."}</p><button className="preview-outline-button" type="button" onClick={() => void refresh()}>RETRY ↗</button></div>;
  return <>{children}</>;
}
