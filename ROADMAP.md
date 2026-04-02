# Roadmap R2

> **Version courante** : 0.1.5 | **Statut** : Phase 1 - Foundation

---

## 📍 Vue d'Ensemble

```
Phase 1: Foundation    (Mois 1-3)   ████████████░░░░░░░░░░░░░░░░░░░
Phase 2: Core Gameplay (Mois 4-6)   ░░░░░░░░░░░░░░█████████████░░░░░░
Phase 3: Economy       (Mois 7-9)   ░░░░░░░░░░░░░░░░░░░░░░░░░░░██████
Phase 4: Content       (Mois 10+)   ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
```

---

## 🚀 Phase 1: Foundation (Mois 1-3)

### Objectif
Infrastructure technique solide, premiers pas multijoueur.

### Deliverables
- [x] Workspace Cargo avec 3 crates
- [x] Types partagés (r2_core)
- [x] Auth OAuth2 + JWT (Discord, GitHub)
- [x] WebSocket bidirectionnel
- [ ] Mouvement multijoueur fluide
- [ ] Zones spatiales

### Progression
| Semaine | Deliverable | Status |
|---------|-------------|--------|
| 1-4 | Workspace + Auth + WebSocket | ✅ v0.1.5 |
| 5-8 | Mouvement multijoueur + Zones | 🔄 En cours |
| 9-12 | Combat basique + PNJ + Persistance | 🔲 Todo |

---

## 🎮 Phase 2: Core Gameplay (Mois 4-6)

### Objectif
Gameplay complet PvP + PVE fun.

### Deliverables
- [ ] Système de skills (8 slots)
- [ ] Types de dégâts (Kinetic, Thermal, EM, Explosive)
- [ ] IA variée (Fighter, Cruiser, Boss)
- [ ] Missions et progression
- [ ] Modes PvP (Arena, Deathmatch)

### Milestones
| Semaine | Deliverable |
|---------|-------------|
| 13-16 | Combat amélioré + Effets visuels |
| 17-20 | Contenu PVE (ennemis, missions) |
| 21-24 | Systèmes PvP + Matchmaking |

---

## 💰 Phase 3: Economy (Mois 7-9)

### Objectif
Économie persistante, crafting, market.

### Deliverables
- [ ] Système d'items avec rarités
- [ ] Inventaire drag & drop
- [ ] Crafting avec recettes
- [ ] Mining et refining
- [ ] Market player-to-player

### Milestones
| Semaine | Deliverable |
|---------|-------------|
| 25-28 | Items + Inventaire + Equipment |
| 29-32 | Crafting + Mining |
| 33-36 | Market + Trading |

---

## 🌍 Phase 4: Content & Social (Mois 10+)

### Objectif
Rétention, communauté, contenus.

### Deliverables
- [ ] Guildes avec wars
- [ ] Plus de vaisseaux (tiers 2-5)
- [ ] World events dynamiques
- [ ] Battle pass saisonnier
- [ ] Launch stable

---

## 📊 Étapes Clés

| Milestone | Version | Status |
|-----------|---------|--------|
| M0 - Prototype | v0.1.0 | ✅ Done |
| M1 - Multiplayer | v0.1.5 | 🔄 En cours |
| M2 - First Blood | v0.2.0 | 🔲 Todo |
| M3 - Combat Fun | v0.3.0 | 🔲 Todo |
| M4 - Content | v0.5.0 | 🔲 Todo |
| M5 - Economy | v0.8.0 | 🔲 Todo |
| M6 - Launch Ready | v1.0.0 | 🔲 Todo |

---

## 🔜 Prochaines Étapes (v0.2.0)

1. Synchronisation du mouvement multijoueur via WebSocket
2. Implémentation des zones spatiales (AOI)
3. Système de collision basique
4. Premier système de combat (tir, dégâts)

---

## 📝 Versions Récentes

### v0.1.5 (2026-04-02)
- OAuth2 GitHub + Email/Password
- Game page avec Canvas 2D
- Correction bug WebSocket (tagged enums)
- Fix boucle infinie render

### v0.1.4 (2026-04-01)
- OAuth2 Discord
- JWT tokens

### v0.1.0 (2026-03-20)
- Initial release
- Bevy ECS integration

---

*Pour les détails techniques, voir [docs/07-roadmap.md](./docs/07-roadmap.md)*
