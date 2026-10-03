import { RoutePlaceholder } from "@/components/route-placeholder";
import { PRODUCT_CONFIG } from "@/config/product";

export default function Home() {
  return <RoutePlaceholder title={PRODUCT_CONFIG.name} description={PRODUCT_CONFIG.description} />;
}
