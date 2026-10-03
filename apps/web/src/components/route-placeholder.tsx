export function RoutePlaceholder({ title, description }: { title: string; description: string }) {
  return (
    <section>
      <h1 className="text-2xl font-semibold">{title}</h1>
      <p className="mt-3 text-slate-600">{description}</p>
      <p className="mt-6 text-sm text-slate-500">Scaffold only. Wallet and protocol actions are not connected yet.</p>
    </section>
  );
}
