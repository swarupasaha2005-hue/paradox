import type { Metadata } from "next";
import { PRODUCT_CONFIG } from "@/config/product";
import "./globals.css";
import "./app.css";
import "./responsive.css";

export const metadata: Metadata = {
  title: { default: PRODUCT_CONFIG.name, template: `%s | ${PRODUCT_CONFIG.name}` },
  description: PRODUCT_CONFIG.description,
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
