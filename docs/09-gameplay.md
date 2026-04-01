# Gameplay R2 - Tactical Action MOBA in Space

> **Version** : 0.1.0 | **Dernière mise à jour** : 2025-04-01
> **Style** : Top-Down Tactical à la Eve/Darkorbit × League of Legends

---

## 🎮 Philosophie du Gameplay

R2 n'est pas un simulation spatial "réaliste". C'est un **action game tactique** où :
- Le mouvement est fluide et précis (comme un MOBA)
- Les armes sont des "skills" à la League of Legends
- L'esquive et le positionnement sont essentiels
- Les vaisseaux ont des profils de jeu distincts

```
┌─────────────────────────────────────────────────────────────────┐
│                    R2 COMBAT PHILOSOPHY                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   "Ce n'est pas le vaisseau qui fait le pilote,                │
│    c'est sa capacité à lire le field et à outplay."            │
│                                                                 │
│   PRIORITÉS:                                                    │
│   1. Mouvement fluide & réactif                                │
│   2. Skills avec impact visuel fort                            │
│   3. Outplay par mécanique, pas par stats                      │
│   4. Teamplay & coordination                                    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🕹️ Contrôles

### Mouvement

```
Clic Droit = Déplacement
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│                    ┌──────┐                                    │
│                    │      │         ← Cible                     │
│                    │  ⛵   │ ─────────────→                     │
│                    │      │                                    │
│                    └──────┘                                    │
│                                                                 │
│   Le vaisseau s'oriente automatiquement vers la cible          │
│   et se déplace avec une légère inertie (feel satisfaisant)    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Caractéristiques :**
- **Accélération progressive** : Pas de démarrage instantané
- **Inertie** : Le vaisseau continue un peu après l'arrêt du input
- **Rotation auto** : Le vaisseau fait face à sa destination
- **Pas de collision physique entre vaisseaux** : Pour éviter le griefing

### Skills

```
Q W E R = Skills principaux
A S D F = Skills secondaires / Utilitaires

[Q]        [W]        [E]        [R]
┌─────┐   ┌─────┐   ┌─────┐   ┌─────┐
│ 1   │   │ 2   │   │ 3   │   │ ULT │
│  3s │   │  8s │   │ 12s │   │ 60s │
└─────┘   └─────┘   └─────┘   └─────┘
```

**Casting :**
- **Instant** : La plupart des skills
- **Channeled** : Maintenir pour continuer (ex: laser continu)
- **Skillshot** : Trajectoire à viser (comme League)
- **Targeted** : Clic sur une cible obligatoire

---

## ⚔️ Les Armes = Skills (MOBA Style)

### Concept

Dans R2, les "armes" ne sont pas des auto-attacks. Ce sont des **skills tactiques** avec :

- **Cooldowns** : Gestion de ressources
- **Cost en énergie** : Mana bar du jeu
- **Effects variés** : Dégâts, CC, buffs, déplacement
- **Améliorations** : Modifier les skills avec des modules

### Exemples de Skills

#### 1. Laser Pulse (Q) - Skillshot linéaire
```
╔════════════════════════════════════════════════════════════════╗
║  🔫 LASER PULSE                                                 ║
║  ────────────────────────────────────────────────────────────  ║
║  Type: Skillshot linéaire                                       ║
║  Cooldown: 3 secondes                                           ║
║  Cost: 15 énergie                                               ║
║                                                                ║
║  Tire un laser qui traverse les ennemis sur 800 unités.        ║
║  Dégâts: 80 + 150% AD                                          ║
║                                                                ║
║  ┌──────────────────────────────────────┐                      ║
║  │  ⛵ ───────────────────────► 💥      │                      ║
║  └──────────────────────────────────────┘                      ║
╚════════════════════════════════════════════════════════════════╝
```

**Améliorations possibles :**
- +Portée
- +Vitesse de projectile
- Réduit cooldown
- Perce blindage

#### 2. Afterburners (W) - Self-buff déplacement
```
╔════════════════════════════════════════════════════════════════╗
║  🚀 AFTERBURNERS                                                ║
║  ────────────────────────────────────────────────────────────  ║
║  Type: Self-buff                                                ║
║  Cooldown: 8 secondes                                           ║
║  Cost: 25 énergie                                               ║
║  Durée: 2 secondes                                              ║
║                                                                ║
║  Augmente la vitesse de 80% et permet de franchir les         ║
║  obstacles/ennemis (fantôme).                                   ║
║                                                                ║
║      💨                                                        ║
║    ⛵──►  (speed +80%)                                          ║
║      💨                                                        ║
╚════════════════════════════════════════════════════════════════╝
```

**Améliorations possibles :**
- +Durée
- Confère un shield pendant l'activation
- Laisse une traîne qui brûle les ennemis

#### 3. EMP Burst (E) - AOE autour du vaisseau
```
╔════════════════════════════════════════════════════════════════╗
║  💥 EMP BURST                                                   ║
║  ────────────────────────────────────────────────────────────  ║
║  Type: AOE autour du vaisseau                                   ║
║  Cooldown: 12 secondes                                          ║
║  Cost: 40 énergie                                               ║
║                                                                ║
║  Émet une impulsion EM qui étourdit les ennemis à proximité.   ║
║  Range: 300 unités                                              ║
║  Stun: 1.5 seconde                                              ║
║                                                                ║
║         👻 👻                                                   ║
║      👻  ⛵  👻                                                 ║
║         👻 👻                                                   ║
╚════════════════════════════════════════════════════════════════╝
```

#### 4. Warp Disruptor (R) - Ultimate
```
╔════════════════════════════════════════════════════════════════╗
║  🌀 WARP DISRUPTOR (Ultimate)                                   ║
║  ────────────────────────────────────────────────────────────  ║
║  Type: Skillshot chargé                                         ║
║  Cooldown: 60 secondes                                          ║
║  Cost: 100 énergie                                              ║
║                                                                ║
║  Après 2s de cast, téléporte le vaisseau et inflige de         ║
║  lourds dégâts à l'arrivée.                                    ║
║                                                                ║
║  [CAST]                [TÉLÉPORTATION]                          ║
║    ⛵ 💫      ──────────────────►  💥 ⛵ 💥                    ║
║                  (deg zone)                                     ║
╚════════════════════════════════════════════════════════════════╝
```

---

## 🚢 Profils de Vaisseaux

### Fighter - L'Assassin

```
┌─────────────────────────────────────────────────────────────────┐
│  ⚔ FIGHTER                                                     │
├─────────────────────────────────────────────────────────────────┤
│  Style: Hit & Run                                              │
│                                                                │
│  FORCES:                                                        │
│  + Vitesse très élevée (200 base)                              │
│  + Accélération réactive                                       │
│  + Petit hitbox (dur à toucher)                                │
│                                                                │
│  FAIBLESSES:                                                    │
│  - Très fragile                                                │
│  - Portée limitée                                             │
│  - Dépend du outplay                                          │
│                                                                │
│  RÔLE: Flank, Assassination, Harassment                        │
│                                                                │
│  "Je ne suis pas là, et déjà tu es mort."                       │
└─────────────────────────────────────────────────────────────────┘
```

### Frigate - L'Équilibré

```
┌─────────────────────────────────────────────────────────────────┐
│  ⚓ FRIGATE                                                     │
├─────────────────────────────────────────────────────────────────┤
│  Style: Bruiser                                                 │
│                                                                │
│  FORCES:                                                        │
│  + Stats équilibrées                                          │
│  + 3 weapon slots                                             │
│  + Polyvalent                                                  │
│                                                                │
│  FAIBLESSE:                                                    │
│  - Pas de spécialité                                          │
│  - Outclassé en 1v1 par spécialistes                          │
│                                                                │
│  RÔLE: Frontline, Support, Flexible                            │
│                                                                │
│  "Je peux tout faire, mais rien parfaitement."                   │
└─────────────────────────────────────────────────────────────────┘
```

### Cruiser - Le Tank

```
┌─────────────────────────────────────────────────────────────────┐
│  🛡️ CRUISER                                                     │
├─────────────────────────────────────────────────────────────────┤
│  Style: Anchor / Zone Control                                   │
│                                                                │
│  FORCES:                                                        │
│  + HP massif                                                   │
│  + Shield solide                                               │
│  + 4 weapon slots                                             │
│  + Contrôle de zone                                            │
│                                                                │
│  FAIBLESSE:                                                    │
│  - Lent                                                        │
│  - Peu manouvrable                                            │
│  - Vulnérable au kite                                          │
│                                                                │
│  RÔLE: Tank, Frontline, Zone Control                           │
│                                                                │
│  "Viens me faire. Je te garantis que tu vas regretter."         │
└─────────────────────────────────────────────────────────────────┘
```

### Hauler - Le Support

```
┌─────────────────────────────────────────────────────────────────┐
│  📦 HAULER                                                      │
├─────────────────────────────────────────────────────────────────┤
│  Style: Utility / Economy                                       │
│                                                                │
│  FORCES:                                                        │
│  + Cargo massif (500 slots)                                    │
│  + Skills utilitaires puissants                                │
│  + Peu ciblé en PvP (full loot = pas de cargo)                │
│                                                                │
│  FAIBLESSE:                                                    │
│  - Zéro combatif                                              │
│  - Fuit si aggressé                                           │
│                                                                │
│  RÔLE: Mining, Trading, Logistique                             │
│                                                                │
│  "Je ne combats pas. Je te rends riche, ou je fuis."           │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🎯 Système de Stats

### Stats de Vaisseau

| Stat | Impact |
|------|--------|
| **Hull** | Points de vie "permanents" |
| **Shield** | PV régénératifs, cassent puis slow regen |
| **Energy** | Ressource pour cast les skills |
| **Speed** | Vitesse de déplacement max |
| **Acceleration** | Vitesse pour atteindre max speed |
| **Turn Rate** | Vitesse de rotation |

### Stats d'Armes (Skills)

| Stat | Impact |
|------|--------|
| **Damage** | Dégâts bruts |
| **AD Ratio** | % d'une stat de vaisseau ajoutée |
| **Range** | Portée du skill |
| **Cooldown** | Temps entre utilisations |
| **Cast Time** | Temps avant l'effet |
| **Energy Cost** | Coût en énergie |

---

## 🔧 Système d'Amélioration

### Modules

Chaque skill peut être amélioré avec des modules :

```
┌─────────────────────────────────────────────────────────────────┐
│  MODULE: LENS FOCUSING                                         │
├─────────────────────────────────────────────────────────────────┤
│  +20% Range sur les skills linéaires                          │
│  -5% Vitesse de projectile                                    │
│                                                                │
│  Compatible avec: Laser Pulse, Railgun Shot                   │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  MODULE: QUICK CHARGE                                          │
├─────────────────────────────────────────────────────────────────┤
│  -15% Cooldown sur tous les skills                            │
│  +10% Energy cost                                              │
│                                                                │
│  Compatible avec: Tous                                         │
└─────────────────────────────────────────────────────────────────┘
```

### Ship Loadout

```
SLOTS:  [W] [W] [W] [U] [U] [P] [P] [P]

W = Weapon skill module
U = Utility skill module
P = Passive module
```

---

## 💡 Principes de Design

### 1. Le Mouvement est Roi

> *"Un joueur qui bouge bien vaut 10 joueurs qui ont des skills plus forts mais ne bougent pas."*

- **Esquive active** : Bouger pendant le cast d'un skill ennemi peut le faire rater
- **Kiting** : Hit and run pour les rapides
- **Positioning** : Être au bon moment au bon endroit

### 2. Clear Input = Clear Output

- **Visual feedback immédiat** : Chaque action a une réponse visuelle
- **Audio feedback** : Sons distincts par type de skill
- **Hit confirmation** : Savoir si on a touché

### 3. Depth over Complexity

- **Peu de skills** (8 max) mais chacun profond
- **Combinaisons** : Skills synergisent entre eux
- **Outplay potentiel** : Mécaniques qui récompensent la maîtrise

---

## 🎓 Exemples de Combos

### Fighter Combo
```
[W] Afterburners (engagement)
  ↓
[Q] Laser Pulse (dégâts entrée)
  ↓
[E] EMP Burst (stun si proximité)
  ↓
[Q] Laser Pulse (finish)
  ↓
[E] Afterburners (disengage)
```

### Frigate Combo
```
[W] Shield Boost (tank)
  ↓
[E] Concussion Missile (CC)
  ↓
[Q] Auto-cannons (burst)
  ↓
[R] Orbital Strike (exécution)
```

---

*Gameplay v0.1.0 - Sujet à ajustement selon playtests*
