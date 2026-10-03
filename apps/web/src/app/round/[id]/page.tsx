import { RoutePlaceholder } from "@/components/route-placeholder";

export default async function RoundPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  return <RoutePlaceholder title={`Financing round ${id}`} description="Commit, reveal, verify, allocate, and settle financing." />;
}
