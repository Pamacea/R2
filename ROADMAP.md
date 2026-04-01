# Roadmap R2

> **Version** : 0.1.0 | **Statut** : En développement

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
- [ ] Workspace Cargo avec 3 crates
- [ ] Types partagés (r2_core)
- [ ] Auth OAuth2 + JWT
- [ ] WebSocket bidirectionnel
- [ ] Mouvement multijoueur fluide
- [ ] Zones spatiales

### Milestones
| Semaine | Deliverable |
|---------|-------------|
| 1-4 | Workspace + Auth + WebSocket |
| 5-8 | Mouvement multijoueur + Zones |
| 9-12 | Combat basique + PNJ + Persistance |

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

| Milestone | Date Cible | Status |
|-----------|-----------|--------|
| M0 - Prototype | S4 | 🔲 Todo |
| M1 - Multiplayer | S8 | 🔲 Todo |
| M2 - First Blood | S12 | 🔲 Todo |
| M3 - Combat Fun | S16 | 🔲 Todo |
| M4 - Content | S24 | 🔲 Todo |
| M5 - Economy | S36 | 🔲 Todo |
| M6 - Launch Ready | S48 | 🔲 Todo |

---

## 🔜 Prochaines Étapes

1. Créer le workspace Cargo
2. Implémenter `r2_core` avec types de base
3. Setup `r2_server` avec Axum
4. Setup `r2_client` avec Bevy
5. Première connection WebSocket

---

*Pour les détails techniques, voir [docs/07-roadmap.md](./docs/07-roadmap.md)*
