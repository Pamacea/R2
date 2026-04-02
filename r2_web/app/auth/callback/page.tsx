"use client";

import { useEffect } from "react";

export default function AuthCallbackPage() {
  useEffect(() => {
    // Le AuthProvider va automatiquement détecter les paramètres OAuth
    // (code, state) dans l'URL et échanger le code contre des tokens
    // Une fois authentifié, on redirige vers la page d'accueil
    const timer = setTimeout(() => {
      window.location.href = "/";
    }, 1000);

    return () => clearTimeout(timer);
  }, []);

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-900">
      <div className="text-center">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500 mx-auto mb-4"></div>
        <p className="text-gray-400">Connexion en cours...</p>
      </div>
    </div>
  );
}
