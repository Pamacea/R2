# R2 - MMO Spatial Web

> **Version** : 0.1.0 | **Statut** : Pré-Alpha | **Stack** : Rust + Bevy + WebGPU

---

## 🚀 Vision

R2 est un MMO spatial jouable entièrement dans le navigateur, combinant :
- **Combat PvP** intense (espaces instables)
- **Exploration PVE** (zones sécurisées, anomalies, ressources)
- **Économie persistante** (crafting, trading, marché)
- **Progression** (skills, vaisseaux, équipement)

**Inspiration** : *EVE Online* (économie/depth) × *V Rising* (gameplay fluide) × *Albion* (full-loot PvP)

---

## 🎯 Objectifs du Projet

### Phase 1 : Fondations (Mois 1-3)
- [ ] Architecture client/serveur avec Bevy headless partagé
- [ ] Système de networking WebSocket + Spatial Hashing
- [ ] Auth complète (OAuth2 + JWT)
- [ ] Mouvement multiplayer fluide avec interpolation

### Phase 2 : Core Gameplay (Mois 4-6)
- [ ] Système de combat avec skills
- [ ] AI des ennemis PVE
- [ ] Premières zones (Safe zone + Zone PvP)
- [ ] UI complète (Dioxus overlay)

### Phase 3 : Économie & Persistance (Mois 7-9)
- [ ] Inventaire et équipement
- [ ] Système de crafting
- [ ] Marché player-to-player
- [ ] Clans/Guildes

### Phase 4 : Scaling (Mois 10+)
- [ ] Cluster de serveurs
- [ ] Load balancing
- [ ] Anti-cheat

---

## 📁 Structure du Projet

```
R2/
├── docs/              # Documentation (ce dossier)
├── r2_core/           # Code partagé client/serveur
│   ├── src/
│   │   ├── components/   # Composants ECS partagés
│   │   ├── network/      # Protocole de messages
│   │   └── types/        # Types de domaine
│   └── Cargo.toml
├── r2_server/         # Serveur de simulation
│   ├── src/
│   │   ├── ecs/          # Systèmes serveur
│   │   ├── spatial/      # Spatial hashing
│   │   ├── db/           # PostgreSQL
│   │   └── auth/         # OAuth2/JWT
│   └── Cargo.toml
├── r2_client/         # Client Bevy Web
│   ├── src/
│   │   ├── engine/       # Systèmes Bevy
│   │   ├── ui/           # Bridge Dioxus
│   │   └── network/      # Client WebSocket
│   └── Cargo.toml
└── Cargo.toml         # Workspace root
```

---

## 🛠️ Stack Technique

| Composant | Technologie | Justification |
|-----------|-------------|---------------|
| **Moteur Jeu** | Bevy 0.15+ | ECS moderne, WebGPU natif, Wasm-friendly |
| **UI Overlay** | Dioxus | React-like en Rust, isomorphique web/desktop |
| **Serveur Web** | Axum 0.7+ | Performant, async, Tokio-based |
| **ECS Serveur** | Bevy Headless | Code partagé avec client |
| **Base de Données** | PostgreSQL 16+ | Persistance fiable, JSONB pour les composants |
| **Cache** | Redis | Sessions, market data, leaderboards |
| **Protocole** | WebSocket + Serde | Temps réel, typesafe |
| **Auth** | OAuth2 + JWT | Google/Discord login |
| **Déploiement** | Vercel/Clever Cloud | Web natif |

---

## 📖 Documentation

- [Architecture](./01-architecture.md) - Vue d'ensemble technique
- [Networking](./02-networking.md) - Protocole et communication
- [ECS Design](./03-ecs.md) - Entity Component System
- [Database Schema](./04-database.md) - Schéma PostgreSQL
- [Auth System](./05-auth.md) - Authentification
- [Performance](./06-performance.md) - Optimisations et scaling
- [Roadmap](./07-roadmap.md) - Plan de développement détaillé
- [DA & UI](./08-da-ui.md) - Direction Artistique & Style Tactical Industrial
- [Gameplay](./09-gameplay.md) - ⭐ Combat MOBA-like, Skills & Vaisseaux
- [Infrastructure](./10-infrastructure.md) - 🚀 Déploiement Railway + Neon + Upstash

---

## 🚀 Quick Start

```bash
# Clone
git clone https://github.com/yourusername/R2.git
cd R2

# Dépendances
cargo install cargo-watch
cargo install wasm-pack

# Serveur (dev)
cd r2_server
cargo run

# Client (dev)
cd r2_client
trunk serve --open
```

---

## 📄 Licence

MIT

---

*Project R2 - Un MMO spatial en Rust, pour le web, par les joueurs.*
