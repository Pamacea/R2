"use client";

/**
 * Contexte d'authentification pour R2
 *
 * Gère OAuth2 (Google, Discord) et les tokens JWT.
 */

import React, { createContext, useContext, useState, useCallback, useEffect } from 'react';

// ============================================================================
// TYPES
// ============================================================================

export type OAuthProvider = 'discord' | 'github';

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  expires_in: number;
  player_id: number;
}

interface AuthState {
  isAuthenticated: boolean;
  playerId: number | null;
  accessToken: string | null;
  refreshToken: string | null;
  tokenExpiresAt: number | null;
  isInitialized: boolean;  // true après le chargement depuis localStorage
}

interface AuthContextValue {
  auth: AuthState;
  login: (provider: OAuthProvider) => void;
  loginWithEmail: (email: string, password: string) => Promise<void>;
  logout: () => void;
  refreshAccessToken: () => Promise<boolean>;
}

// ============================================================================
// CONFIG
// ============================================================================

const API_BASE = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000';

// ============================================================================
// CONTEXT
// ============================================================================

const AuthContext = createContext<AuthContextValue | null>(null);

// ============================================================================
// PROVIDER
// ============================================================================

interface AuthProviderProps {
  children: React.ReactNode;
}

export function AuthProvider({ children }: AuthProviderProps) {
  const [auth, setAuth] = useState<AuthState>({
    isAuthenticated: false,
    playerId: null,
    accessToken: null,
    refreshToken: null,
    tokenExpiresAt: null,
    isInitialized: false,  // Pas encore chargé depuis localStorage
  });

  // Charger les tokens depuis localStorage au démarrage
  useEffect(() => {
    const stored = localStorage.getItem('r2_auth');
    if (stored) {
      try {
        const data = JSON.parse(stored);
        // Vérifier si le token est expiré
        if (data.tokenExpiresAt && Date.now() > data.tokenExpiresAt) {
          // Token expiré, marquer comme initialisé mais pas authentifié
          setAuth({
            isAuthenticated: false,
            playerId: null,
            accessToken: null,
            refreshToken: null,
            tokenExpiresAt: null,
            isInitialized: true,
          });
          // Essayer de rafraîchir en arrière-plan
          refreshAccessToken(data.refresh_token).then(success => {
            if (!success) {
              localStorage.removeItem('r2_auth');
            }
          });
        } else {
          setAuth({
            ...data,
            isInitialized: true,
          });
        }
      } catch (error) {
        console.error('Failed to parse stored auth:', error);
        localStorage.removeItem('r2_auth');
        setAuth(prev => ({ ...prev, isInitialized: true }));
      }
    } else {
      // Pas de données stockées
      setAuth(prev => ({ ...prev, isInitialized: true }));
    }
    if (stored) {
      try {
        const data = JSON.parse(stored);
        // Vérifier si le token est expiré
        if (data.tokenExpiresAt && Date.now() > data.tokenExpiresAt) {
          // Token expiré, essayer de rafraîchir
          refreshAccessToken(data.refresh_token).then(success => {
            if (!success) {
              // Rafraîchissement échoué, déconnecter
              localStorage.removeItem('r2_auth');
              setAuth({
                isAuthenticated: false,
                playerId: null,
                accessToken: null,
                refreshToken: null,
                tokenExpiresAt: null,
                isInitialized: true,
              });
            }
          });
        } else {
          setAuth({ ...data, isInitialized: true });
        }
      } catch (error) {
        console.error('Failed to parse stored auth:', error);
        localStorage.removeItem('r2_auth');
      }
    }
  }, []);

  // Sauvegarder dans localStorage à chaque changement
  useEffect(() => {
    if (auth.isAuthenticated) {
      localStorage.setItem('r2_auth', JSON.stringify(auth));
    } else {
      localStorage.removeItem('r2_auth');
    }
  }, [auth]);

  // Login avec OAuth
  const login = useCallback(async (provider: OAuthProvider) => {
    try {
      // 1. Obtenir l'URL d'autorisation
      const urlResponse = await fetch(`${API_BASE}/auth/url/${provider}`);
      if (!urlResponse.ok) {
        throw new Error('Failed to get auth URL');
      }
      const { auth_url, csrf_token } = await urlResponse.json();

      // Stocker le CSRF token pour vérification plus tard
      sessionStorage.setItem('oauth_csrf', csrf_token);
      sessionStorage.setItem('oauth_provider', provider);

      // 2. Rediriger vers OAuth
      window.location.href = auth_url;
    } catch (error) {
      console.error('Login error:', error);
      throw error;
    }
  }, []);

  // Gérer le callback OAuth (après redirection)
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const code = params.get('code');
    const state = params.get('state');
    const provider = sessionStorage.getItem('oauth_provider') as OAuthProvider | null;
    const csrf = sessionStorage.getItem('oauth_csrf');

    if (code && state && provider && csrf) {
      // Vérifier le CSRF token
      if (state !== csrf) {
        console.error('CSRF token mismatch');
        return;
      }

      // Échanger le code contre des tokens
      exchangeCodeForTokens(provider, code).then(response => {
        if (response) {
          setAuth({
            isAuthenticated: true,
            playerId: response.player_id,
            accessToken: response.access_token,
            refreshToken: response.refresh_token,
            tokenExpiresAt: Date.now() + response.expires_in * 1000,
            isInitialized: true,
          });
          // Nettoyer l'URL
          window.history.replaceState({}, '', window.location.pathname);
        }
      });

      // Nettoyer sessionStorage
      sessionStorage.removeItem('oauth_csrf');
      sessionStorage.removeItem('oauth_provider');
    }
  }, []);

  // Logout
  const logout = useCallback(() => {
    setAuth({
      isAuthenticated: false,
      playerId: null,
      accessToken: null,
      refreshToken: null,
      tokenExpiresAt: null,
      isInitialized: true,
    });
  }, []);

  // Rafraîchir le token d'accès
  const refreshAccessToken = useCallback(async (refreshTokenValue?: string): Promise<boolean> => {
    const token = refreshTokenValue || auth.refreshToken;
    if (!token) return false;

    try {
      const response = await fetch(`${API_BASE}/auth/refresh`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ refresh_token: token }),
      });

      if (!response.ok) {
        return false;
      }

      const data: AuthResponse = await response.json();

      setAuth(prev => ({
        ...prev,
        isAuthenticated: true,
        playerId: data.player_id,
        accessToken: data.access_token,
        refreshToken: data.refresh_token,
        tokenExpiresAt: Date.now() + data.expires_in * 1000,
      }));

      return true;
    } catch (error) {
      console.error('Token refresh error:', error);
      return false;
    }
  }, [auth.refreshToken]);

  // Login avec Email/Password (à implémenter côté backend)
  const loginWithEmail = useCallback(async (email: string, password: string) => {
    try {
      const response = await fetch(`${API_BASE}/auth/email-login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password }),
      });

      if (!response.ok) {
        throw new Error('Login failed');
      }

      const data: AuthResponse = await response.json();

      setAuth({
        isAuthenticated: true,
        playerId: data.player_id,
        accessToken: data.access_token,
        refreshToken: data.refresh_token,
        tokenExpiresAt: Date.now() + data.expires_in * 1000,
        isInitialized: true,
      });
    } catch (error) {
      console.error('Email login error:', error);
      throw error;
    }
  }, []);

  const value: AuthContextValue = {
    auth,
    login,
    loginWithEmail,
    logout,
    refreshAccessToken,
  };

  return (
    <AuthContext.Provider value={value}>
      {children}
    </AuthContext.Provider>
  );
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

async function exchangeCodeForTokens(provider: OAuthProvider, code: string): Promise<AuthResponse | null> {
  try {
    const response = await fetch(`${API_BASE}/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ provider, code }),
    });

    if (!response.ok) {
      console.error('Token exchange failed');
      return null;
    }

    return await response.json();
  } catch (error) {
    console.error('Token exchange error:', error);
    return null;
  }
}

// ============================================================================
// HOOK
// ============================================================================

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within AuthProvider');
  }
  return context;
}
