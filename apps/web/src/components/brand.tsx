import Link from "next/link";

export function Brand({ inverse = false }: { inverse?: boolean }) {
  return (
    <Link href="/" className={`brand${inverse ? " brand--inverse" : ""}`} aria-label="Arth home">
      <span className="brand__mark" aria-hidden="true"><i /><i /><i /></span>
      <span>ARTH<span className="brand__period">.</span></span>
    </Link>
  );
}
