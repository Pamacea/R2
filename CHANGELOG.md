# Changelog

All notable changes to R2 will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- WebSocket networking implementation
- Bevy client rendering
- Axum server simulation
- PostgreSQL deployment
- OAuth integration

---

## [0.1.0] - 2025-04-01

### RELEASE: R2 - v0.1.0

### Added
- Workspace Cargo avec 3 crates (r2_core, r2_server, r2_client)
- Types partagés ECS (Position, Rotation, Velocity, Stats, ShipData, CombatState)
- Protocole réseau WebSocket (R2Message avec snapshots, updates, combat events)
- Système d'authentification JWT + OAuth2 (Google, Discord)
- Repository PostgreSQL (players, characters, inventories, market)
- Spatial Hashing pour Area of Interest
- Configuration Railway pour déploiement

### Documentation
- Architecture technique complète
- Protocole de communication
- Design ECS détaillé
- Schéma de base de données PostgreSQL
- Système d'authentification
- Stratégie de performance et scaling
- Roadmap 12 mois
- Direction Artistique Tactical Industrial
- Gameplay MOBA-like (skills, vaisseaux, combos)
- Infrastructure Railway + Neon + Upstash

### Infrastructure
- **Render.com** configuré pour déploiement (100% gratuit permanent)
- render.yaml pour déploiement automatique (backend + frontend + DB + Redis)
- Variables d'environnement définies
- Stratégie free tier → scale
- GUIDE_RENDER.md pour setup complet

### Changed
- Initial setup from scratch

---

---

## Version Schema

- **Major** (X.0.0) - Breaking changes, major milestones
- **Minor** (0.X.0) - New features, phases
- **Patch** (0.0.X) - Bug fixes, hotfixes

---

*Project R2 - Spatial MMO in Rust*
