import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import { AuthProvider } from "@/contexts/auth";
import { WebSocketProvider } from "@/contexts/websocket";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "R2 - MMO Spatial",
  description: "Explorez l'espace, combattez les ennemis, construisez votre empire",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="fr">
      <body className={inter.className}>
        <AuthProvider>
          <WebSocketProvider>
            {children}
          </WebSocketProvider>
        </AuthProvider>
      </body>
    </html>
  );
}
