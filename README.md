# R2 - Spatial MMO Web

> **Version** : 0.1.0 | **Statut** : En développement
> **Stack** : Rust + Bevy + WebGPU | **Style** : Tactical Industrial

---

## 🚀 Qu'est-ce que R2 ?

R2 est un MMO spatial jouable entièrement dans le navigateur, combinant :
- **Combat PvP** intense (zones instables)
- **Exploration PVE** (zones sécurisées, anomalies)
- **Économie persistante** (crafting, trading, marché)

**Inspiré par** : EVE Online × V Rising × Albion Online

---

## 🛠️ Stack Technique

| Composant | Technologie |
|-----------|-------------|
| Moteur de jeu | [Bevy 0.15](https://bevyengine.org/) (ECS, WebGPU) |
| UI | [Dioxus](https://dioxuslabs.com/) (React-like en Rust) |
| Serveur Web | [Axum](https://github.com/tokio-rs/axum) (async) |
| ECS Serveur | Bevy headless (code partagé) |
| Base de données | PostgreSQL 16+ |
| Cache | Redis |
| Auth | OAuth2 (Google/Discord) + JWT |
| Protocole | WebSocket |

---

## 📁 Structure

```
R2/
├── docs/              # Documentation complète
├── r2_core/           # Code partagé client/serveur
├── r2_server/         # Serveur de simulation
├── r2_client/         # Client Bevy Web
├── Cargo.toml         # Workspace root
├── CHANGELOG.md       # Historique des versions
├── ROADMAP.md         # Feuille de route
└── README.md          # Ce fichier
```

---

## 🚀 Quick Start

### Option 1 : Docker (recommandé - Jouer avec ses potes)

```bash
# Un seul commande !
docker-compose up -d
```

Voir [GUIDE_LOCAL.md](./GUIDE_LOCAL.md) pour les détails.

### Option 2 : Lancer en local (dev)
```

#### Lancer le serveur

```bash
# Copier .env
cp .env.example .env

# Lancer PostgreSQL (optionnel, ou utiliser Render)
docker run -d -p 5432:5432 -e POSTGRES_DB=r2 postgres:16

# Lancer le serveur
cd r2_server
cargo run
```

#### Lancer le client (natif)

```bash
cd r2_client
cargo run
```

#### Lancer le client (web)

```bash
# Installer trunk
cargo install trunk

cd r2_client
trunk serve --open
```

---

## 👥 Jouer avec ses potes

Voir [GUIDE_LOCAL.md](./GUIDE_LOCAL.md) pour :
- Lancer le serveur avec Docker
- Inviter tes potes (local ou via ngrok)

---

## 📖 Documentation

- [Architecture](./docs/01-architecture.md) - Vue d'ensemble technique
- [Networking](./docs/02-networking.md) - Protocole WebSocket
- [ECS Design](./docs/03-ecs.md) - Entity Component System
- [Database Schema](./docs/04-database.md) - Schéma PostgreSQL
- [Auth System](./docs/05-auth.md) - Authentification
- [Performance](./docs/06-performance.md) - Optimisations
- [Roadmap](./docs/07-roadmap.md) - Plan de développement
- [DA & UI](./docs/08-da-ui.md) - Style Tactical Industrial

---

## 🎮 Gameplay (Cible)

### Mouvement
- Clic droit pour déplacer
- Inertie du vaisseau
- Rotation automatique vers la cible

### Combat
- 8 slots de skills
- Cooldowns et coût en énergie
- Types de dégâts : Kinetic, Thermal, EM, Explosive

### Zones
- **Safe Zones** : Pas de PvP, stations
- **PvP Zones** : Full loot, danger
- **PVE Zones** : Ennemis, ressources

---

## 📊 Progression

```
Phase 1: Foundation    (Mois 1-3)  ████████████░░░░░░░░░░░░░░░░░░░
Phase 2: Core Gameplay (Mois 4-6)  ░░░░░░░░░░░░░░█████████████░░░░░░
Phase 3: Economy       (Mois 7-9)  ░░░░░░░░░░░░░░░░░░░░░░░░░░░██████
Phase 4: Content       (Mois 10+)  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
```

Voir [ROADMAP.md](./ROADMAP.md) pour les détails.

---

## 🤝 Contribution

Le projet est en développement précoce. Les contributions sont les bienvenues !

1. Fork le projet
2. Créer une branche `git checkout -b feature/xxx`
3. Commit `git commit -m "Feat: xxx"`
4. Push `git push origin feature/xxx`
5. Ouvrir une Pull Request

---

## 📄 Licence

MIT

---

*Project R2 - Un MMO spatial en Rust, pour le web, par les joueurs.*
