# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.5] - 2026-04-02

### Added
- **OAuth2 GitHub** - Ajout de l'authentification via GitHub OAuth2
- **Email/Password Authentication** - Système d'authentification par email/mot de passe (backend prêt)
- **Game Page** - Page `/game` avec rendu Canvas 2D complet
- **Canvas 2D Rendering** - Rendu spatial avec vaisseau joueur, astéroïdes, stations spatiaux, et autres vaisseaux
- **WebSocket Authentication** - Authentification WebSocket via JWT
- **Auth State Initialization** - État `isInitialized` pour éviter les redirections intempestives
- **Interactive Movement** - Clic pour déplacer le vaisseau avec animation fluide

### Changed
- **OAuth Provider** - Remplacement de Google OAuth par GitHub OAuth
- **Message Format** - Format des messages WebSocket désormais compatible avec le client JavaScript (tagged enums avec `serde(tag = "type")`)
- **Auth Callback** - Page `/auth/callback` pour gérer les retours OAuth
- **State Management** - Utilisation de `useRef` pour les valeurs de position évitant les re-renders infinis
- **UI Overlay** - Interface utilisateur avec position joueur, barres de vie (Hull/Energy)

### Fixed
- **CORS Configuration** - Correction de l'ordre des layers Axum (CORS après les routes pour exécution avant routing)
- **Infinite Loop** - Correction de la boucle infinie de re-render dans game/page.tsx (useEffect dependencies)
- **Base64 Encoding** - Mise à jour vers l'API `base64::Engine` (v0.22)
- **OAuth Tests** - Mise à jour des tests pour utiliser GitHub au lieu de Google
- **WebSocket Parsing** - Correction du format des messages (tagged enums)
- **Auth Redirect** - Le GamePage attend maintenant l'initialisation de l'auth avant de rediriger

### Removed
- **Google OAuth** - Suppression de Google OAuth (remplacé par GitHub)
- **Duplicate Files** - Nettoyage des fichiers doublons dans `app/contexts/` et `src/app/`

## [0.1.4] - 2026-04-01

### Added
- **OAuth2 Discord** - Authentification via Discord OAuth2
- **JWT Tokens** - Système de tokens d'accès et de refresh
- **Auth Context** - Contexte React pour la gestion d'authentification

### Changed
- **Token Validation** - Validation des tokens JWT avec expiration

## [0.1.3] - 2026-03-30

### Added
- **WebSocket Handler** - Gestionnaire de connexions WebSocket
- **Player Connection** - Structure de connexion joueur

## [0.1.2] - 2026-03-28

### Added
- **Auth Module** - Module d'authentification JWT
- **OAuth2 Infrastructure** - Structure pour OAuth2

## [0.1.1] - 2026-03-25

### Added
- **Axum Server** - Serveur HTTP avec Axum
- **Database Module** - Module de base de données

## [0.1.0] - 2026-03-20

### Added
- **Initial Release** - Projet R2 MMO Spatial
- **Bevy ECS** - Intégration de Bevy ECS pour la simulation
- **Core Components** - Composants de base du jeu
