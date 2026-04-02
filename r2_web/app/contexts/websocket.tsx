"use client";

/**
 * Contexte WebSocket pour R2
 *
 * Gère la connexion WebSocket avec le serveur Rust et fournit
 * les hooks pour envoyer/recevoir des messages en temps réel.
 */

import React, { createContext, useContext, useEffect, useState, useCallback } from 'react';

// ============================================================================
// TYPES
// ============================================================================

/** Message Client → Serveur */
export type ClientMessage =
  | { type: 'connect'; token: string }
  | { type: 'input'; thrust: boolean; rotate: number; fire: boolean }
  | { type: 'ping'; timestamp: number };

/** Message Serveur → Client */
export type ServerMessage =
  | { type: 'connected'; player_id: number; characters: CharacterSummary[] }
  | { type: 'worldUpdate'; entities: EntityData[]; timestamp: number }
  | { type: 'error'; message: string }
  | { type: 'pong'; timestamp: number };

/** Résumé d'un personnage */
export interface CharacterSummary {
  id: number;
  name: string;
  ship_class: string;
  level: number;
}

/** Données d'une entité */
export interface EntityData {
  id: number;
  entity_type: EntityType;
  position: [number, number, number];
  rotation: number;
  velocity?: [number, number];
  hull_percent: number;
  shield_percent: number;
}

export type EntityType =
  | 'Player'
  | 'NpcHostile'
  | 'NpcFriendly'
  | 'Asteroid'
  | 'Station'
  | 'Loot'
  | 'Projectile';

/** État de la connexion WebSocket */
type ConnectionState = 'connecting' | 'connected' | 'disconnected' | 'error';

// ============================================================================
// CONTEXT
// ============================================================================

interface WebSocketContextValue {
  connectionState: ConnectionState;
  playerId: number | null;
  characters: CharacterSummary[];
  entities: Map<number, EntityData>;
  send: (message: ClientMessage) => void;
  connect: (token: string) => void;
  disconnect: () => void;
}

const WebSocketContext = createContext<WebSocketContextValue | null>(null);

// ============================================================================
// PROVIDER
// ============================================================================

interface WebSocketProviderProps {
  children: React.ReactNode;
  url?: string;
}

export function WebSocketProvider({ children, url = 'ws://localhost:3000/ws' }: WebSocketProviderProps) {
  const [ws, setWs] = useState<WebSocket | null>(null);
  const [connectionState, setConnectionState] = useState<ConnectionState>('disconnected');
  const [playerId, setPlayerId] = useState<number | null>(null);
  const [characters, setCharacters] = useState<CharacterSummary[]>([]);
  const [entities, setEntities] = useState<Map<number, EntityData>>(new Map());

  // Envoyer un message au serveur
  const send = useCallback((message: ClientMessage) => {
    if (ws?.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(message));
    }
  }, [ws]);

  // Connecter avec un token JWT
  const connect = useCallback((token: string) => {
    if (ws?.readyState === WebSocket.OPEN) {
      send({ type: 'connect', token });
    }
  }, [ws, send]);

  // Déconnecter
  const disconnect = useCallback(() => {
    ws?.close();
    setWs(null);
    setConnectionState('disconnected');
    setPlayerId(null);
    setCharacters([]);
    setEntities(new Map());
  }, [ws]);

  // Initialiser la connexion WebSocket
  useEffect(() => {
    const socket = new WebSocket(url);

    socket.onopen = () => {
      console.log('WebSocket connected');
      setConnectionState('connected');
    };

    socket.onclose = () => {
      console.log('WebSocket disconnected');
      setConnectionState('disconnected');
    };

    socket.onerror = (error) => {
      console.error('WebSocket error:', error);
      setConnectionState('error');
    };

    socket.onmessage = (event) => {
      try {
        const message: ServerMessage = JSON.parse(event.data);

        switch (message.type) {
          case 'connected':
            console.log('Authenticated as player', message.player_id);
            setPlayerId(message.player_id);
            setCharacters(message.characters);
            break;

          case 'worldUpdate':
            // Mettre à jour les entités
            setEntities(prev => {
              const next = new Map(prev);
              for (const entity of message.entities) {
                next.set(entity.id, entity);
              }
              return next;
            });
            break;

          case 'error':
            console.error('Server error:', message.message);
            break;

          case 'pong':
            // Ping/pong pour keep-alive
            break;
        }
      } catch (error) {
        console.error('Failed to parse message:', error);
      }
    };

    setWs(socket);

    return () => {
      socket.close();
    };
  }, [url]);

  // Ping périodique pour keep-alive
  useEffect(() => {
    if (connectionState !== 'connected') return;

    const interval = setInterval(() => {
      send({ type: 'ping', timestamp: Date.now() });
    }, 30000); // 30 secondes

    return () => clearInterval(interval);
  }, [connectionState, send]);

  const value: WebSocketContextValue = {
    connectionState,
    playerId,
    characters,
    entities,
    send,
    connect,
    disconnect,
  };

  return (
    <WebSocketContext.Provider value={value}>
      {children}
    </WebSocketContext.Provider>
  );
}

// ============================================================================
// HOOK
// ============================================================================

export function useWebSocket() {
  const context = useContext(WebSocketContext);
  if (!context) {
    throw new Error('useWebSocket must be used within WebSocketProvider');
  }
  return context;
}
