# Roadmap R2

> **Version** : 0.1.0 | **Dernière mise à jour** : 2025-04-01

---

## 🗺️ Vue d'Ensemble

```
Phase 1: Foundation    (Mois 1-3)  ████████████░░░░░░░░░░░░░░░░░░░
Phase 2: Core Gameplay (Mois 4-6)  ░░░░░░░░░░░░░░█████████████░░░░░░
Phase 3: Economy       (Mois 7-9)  ░░░░░░░░░░░░░░░░░░░░░░░░░░░██████
Phase 4: Content       (Mois 10+)  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
```

---

## 📅 Phase 1: Foundation (Mois 1-3)

> **Objectif** : Infrastructure technique solide, premiers pas multijoueur

### Mois 1: Squelette Technique

#### Semaine 1 : Workspace + Shared Code
- [ ] Initialiser workspace Cargo
- [ ] Créer `r2_core` avec types de base
- [ ] Définir `r2_core/src/network/msgs.rs` (enum R2Message)
- [ ] Créer `r2_core/src/components/` (Position, Stats, ShipData)

**Deliverable** : Code partagé compilable client + serveur

#### Semaine 2 : Serveur Axum + Auth
- [ ] Initialiser `r2_server` avec Axum
- [ ] Setup PostgreSQL (Docker compose)
- [ ] Implémenter OAuth2 (Google + Discord)
- [ ] JWT generation/validation
- [ ] API endpoints : `/auth/{provider}`, `/auth/refresh`

**Deliverable** : Login fonctionnel, JWT valide

#### Semaine 3 : Client Bevy de base
- [ ] Initialiser `r2_client` avec Bevy
- [ ] Configurer WebGPU target (Wasm)
- [ ] Caméra top-down fixe
- [ ] Mesh simple (cube) qui bouge au clavier
- [ ] UI Dioxus overlay (Login screen)

**Deliverable** : Cube qui bouge dans le navigateur

#### Semaine 4 : WebSocket Connection
- [ ] WebSocket upgrade côté serveur
- [ ] Client WebSocket connect
- [ ] Heartbeat/Ping-Pong
- [ ] Auth via JWT dans handshake WS

**Deliverable** : Connection persistante client ↔ serveur

### Mois 2: Mouvement Multijoueur

#### Semaine 5 : ECS Serveur
- [ ] Bevy headless côté serveur
- [ ] Système mouvement basique
- [ ] Spatial Hashing pour AOI
- [ ] Loop de simulation (30Hz)

**Deliverable** : Simulation serveur tourne

#### Semaine 6 : Réplication Position
- [ ] `MoveTo` message (client → serveur)
- [ ] Validation mouvement (speed check)
- [ ] `WorldUpdate` message (serveur → client)
- [ ] Interpolation client-side

**Deliverable** : Deux joueurs se voient bouger

#### Semaine 7 : Prédiction Client
- [ ] Client-side prediction pour mouvement local
- [ ] Reconciliation si serveur corrige
- [ ] Lag compensation basique

**Deliverable** : Mouvement fluide même avec 100ms ping

#### Semaine 8 : Zones & Portails
- [ ] Système de zones spatiales
- [ ] Portail pour changer de zone
- [ ] Transition smooth (chargement map)

**Deliverable** : 2 zones accessibles via portail

### Mois 3: Premier Gameplay

#### Semaine 9 : Combat Basique
- [ ] Système de dégâts
- [ ] HP/Shield
- [ ] Laser projectile (visuel)
- [ ] `UseSkill` message

**Deliverable** : On peut tirer

#### Semaine 10 : PNJ & AI
- [ ] IA simple (chase + attack)
- [ ] Spawning PNJ dans zones
- [ ] Loot drop basique

**Deliverable** : PNJ qui attaquent

#### Semaine 11 : UI Complète
- [ ] Barre vie/shield/energy
- [ ] Barre de skills
- [ ] Cible verrouillée
- [ ] Chat basique

**Deliverable** : UI fonctionnelle

#### Semaine 12 : Persistance
- [ ] Création character
- [ ] Sauvegarde position/stats
- [ ] Load au login

**Deliverable** : Logout = save, login = restore

---

## 🎮 Phase 2: Core Gameplay (Mois 4-6)

> **Objectif** : Gameplay complet PvP + PVE fun

### Mois 4: Combat Amélioré

#### Semaine 13 : Système de Skills
- [ ] 8 skills par vaisseau
- [ ] Cooldowns
- [ ] Coût énergie
- [ ] Skill variety (damage, heal, buff, debuff)

#### Semaine 14 : Types de dégâts
- [ ] Kinetic, Thermal, EM, Explosive
- [ ] Résistances par vaisseau
- [ ] Calcul damage final

#### Semaine 15 : Effets Visuels
- [ ] Bloom sur impacts
- [ ] Particules explosions
- [ ] Trail projectiles
- [ ] Screen shake

#### Semaine 16 : Audio
- [ ] Sons tir, impact, explosion
- [ ] Musique dynamic (combat/calm)
- [ ] Spatial audio

### Mois 5: PVE Content

#### Semaine 17 : Types d'ennemis
- [ ] Fighter (rapide, fragile)
- [ ] Cruiser (lent, tanky)
- [ ] Boss unique mechanics

#### Semaine 18 : Missions
- [ ] Système de quêtes
- [ ] Mission types (kill, collect, explore)
- [ ] Rewards (xp, credits, items)

#### Semaine 19 : Zones variées
- [ ] Safe zone (no PvP)
- [ ] PvP zone (full loot)
- [ ] PVE zone (hard NPCs)

#### Semaine 20 : Progression
- [ ] XP system
- [ ] Level up = stat points
- [ ] Unlock skills

### Mois 6: PvP Systems

#### Semaine 21 : Matchmaking
- [ ] Queue solo/duo/squad
- [ ] Match instancié
- [ ] ELO rating

#### Semaine 22 : Arena modes
- [ ] Deathmatch
- [ ] Capture point
- [ : Escort

#### Semaine 23 : Leaderboards
- [ ] Global rankings
- [ ] Seasonal rewards

#### Semaine 24 : Replay system
- [ ] Enregistrer match
- [ ] Replay client-side

---

## 💰 Phase 3: Economy (Mois 7-9)

> **Objectif** : Économie persistante, crafting, market

### Mois 7: Items & Inventaire

#### Semaine 25 : Système d'items
- [ ] Item database
- [ ] Rarity colors
- [ ] Stats randomisés
- [ ] Item tooltips

#### Semaine 26 : Inventory UI
- [ ] Drag & drop
- [ ] Split stacks
- [ ] Sort/filter
- [ ] Capacity management

#### Semaine 27 : Equipment
- [ ] Ship loadout save/load
- [ ] Slots: weapon, utility, passive
- [ ] Visual changes per item

#### Semaine 28 : Loot
- [ ] Loot tables par NPC
- [ ] Loot containers
- [ ] Quality drop chances

### Mois 8: Crafting

#### Semaine 29 : Recettes
- [ ] Crafting tree
- [ ] Required materials
- [ ] Skill level requirement

#### Semaine 30 : Crafting UI
- [ ] Recipe browser
- [ ] Materials check
- [ ] Queue crafting

#### Semaine 31 : Mining
- [ ] Astéroïdes minables
- [ ] Mining laser
- [ ] Rare materials

#### Semaine 32 : Refining
- [ ] Ore → materials
- [ ] Refining stations
- [ ] Efficiency skill

### Mois 9: Market

#### Semaine 33 : Market UI
- [ ] Browse orders
- [ ] Place buy/sell
- [ : Price history graph

#### Semaine 34 : Market backend
- [ ] Order matching
- [ ] Transaction logging
- [ ] Fees/taxes

#### Semaine 35 : Player Trading
- [ ] Direct trade
- [ ] Trade window
- [ ] Secure exchange

#### Semaine 36 : Economy Tuning
- [ ] Gold sinks
- [ ] Faucets
- [ ] Inflation monitoring

---

## 🌍 Phase 4: Content & Social (Mois 10+)

> **Objectif** : Retention, communauté, contenus

### Mois 10: Guildes

#### Semaine 37 : Guild creation
- [ ] Create guild
- [ ] Invite/kick
- [ ] Ranks & permissions

#### Semaine 38 : Guild features
- [ ] Guild chat
- [ ] Guild bank
- [ ] Guild bonuses

#### Semaine 39 : Guild wars
- [ ] Declare war
- [ ] War objectives
- [ ] Rewards

#### Semaine 40 : Guild housing
- [ ] Guild station
- [ ] Upgrades

### Mois 11: Ships & Progression

#### Semaine 41 : More ships
- [ ] New ship classes
- [ ] Tier 2-5 ships
- [ ] Ship mastery

#### Semaine 42 : Ship customization
- [ ] Colors
- [ ] Decals
- [ ] Titles

#### Semaine 43 : Skill tree
- [ ] Passive skills
- [ ] Active skills unlock
- [ ] Respec

#### Semaine 44 : Endgame gear
- [ ] Legendary items
- [ ] Set bonuses
- [ ] Raid drops

### Mois 12: World Events

#### Semaine 45 : Dynamic events
- [ ] Invasion events
- [ : World bosses
- [ ] Community goals

#### Semaine 46 : Daily/Weekly
- [ ] Login rewards
- [ ] Daily quests
- [ ] Weekly challenges

#### Semaine 47 : Seasonal
- [ ] Battle pass
- [ ] Season rewards
- [ : Seasonal events

#### Semaine 48 : Analytics
- [ ] Player tracking
- [ ] Retention metrics
- [ ] Balance tuning

---

## 🚀 Post-Launch

#### Semaine 49+ : Live Ops
- [ ] Hotfix system
- [ ] Patch notes
- [ ] Community management
- [ ] Anti-cheat improvements
- [ ] Content updates

---

## 📊 Milestones

| Milestone | Date cible | Critères de succès |
|-----------|-----------|-------------------|
| **M0 - Prototype** | Semaine 4 | Connection WS, mouvement cube |
| **M1 - Multiplayer** | Semaine 8 | 2 joueurs se voient bouger |
| **M2 - First Blood** | Semaine 12 | Combat PNJ fonctionnel |
| **M3 - Combat Fun** | Semaine 16 | PvP enjoyable |
| **M4 - Content** | Semaine 24 | 3 zones, 5 types ennemis |
| **M5 - Economy** | Semaine 36 | Market actif, crafting |
| **M6 - Launch Ready** | Semaine 48 | Full game, stable |

---

## 🎯 Definition of Done

Par **feature** :

```markdown
## [Feature Name]

### Code
- [ ] Implémenté
- [ ] Testé (unit tests si applicable)
- [ ] Linté (clippy clean)
- [ ] Documenté (comments)

### Gameplay
- [ ] Fun à jouer
- [ ] Feedback visuel/auditif
- [ ] Équilibré (valeurs raisonnables)

### Network
- [ ] Latence acceptable (<100ms perçu)
- [ ] Reconnexion handle proprement
- [ ] Pas de désync majeur

### UI/UX
- [ ] Intuitif
- [ ] Tooltips/help
- [ ] Responsive (different screens)
```

---

## 🔄 Iteration Speed

### Sprints de 1 semaine
- Lundi : Planning
- Mardi-Mercredi : Dev
- Jeudi : Review + QA
- Vendredi : Bugfix + Deploy

### Playtests bi-hebdomadaires
- Mercredi : Interne
- Samedi : Alpha testers

---

*Roadmap v0.1.0 - Sujet à ajustement*
