# Architecture R2

> **Version** : 0.1.0 | **Dernière mise à jour** : 2025-04-01

---

## 📐 Vue d'Ensemble

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CLUSTERS R2                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐                    │
│  │   Cluster   │    │   Cluster   │    │   Cluster   │                    │
│  │   Alpha     │    │   Beta      │    │   Gamma     │                    │
│  │ (Zone 1-10) │    │ (Zone 11-20)│    │ (Zone 21-30)│                    │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘                    │
│         │                  │                  │                            │
│         └──────────────────┼──────────────────┘                            │
│                            │                                               │
│                    ┌───────▼────────┐                                     │
│                    │  Load Balancer │                                     │
│                    │   (Envoy)      │                                     │
│                    └───────┬────────┘                                     │
│                            │                                               │
└────────────────────────────┼───────────────────────────────────────────────┘
                             │
                    ┌────────▼────────┐
                    │  API Gateway    │
                    │  (Axum + Auth)  │
                    └────────┬────────┘
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
┌───────▼────────┐  ┌────────▼────────┐  ┌───────▼────────┐
│   PostgreSQL   │  │     Redis       │  │  RabbitMQ/     │
│   (Persistance)│  │   (Cache/State) │  │  NATS (Events) │
└────────────────┘  └─────────────────┘  └────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                              CLIENTS WEB                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────┐       │
│  │                    Browser (Chrome/Firefox)                     │       │
│  ├─────────────────────────────────────────────────────────────────┤       │
│  │                                                                  │       │
│  │  ┌─────────────────┐    ┌─────────────────┐                    │       │
│  │  │   Dioxus UI     │    │  Bevy Canvas     │                    │       │
│  │  │  (HTML/CSS)     │    │  (WebGPU/Wasm)   │                    │       │
│  │  │                 │    │                  │                    │       │
│  │  │ • Inventory     │    │ • 3D Rendering   │                    │       │
│  │  │ • Chat          │    │ • Particles      │                    │       │
│  │  │ • Market        │    │ • Ships          │                    │       │
│  │  │ • Skills        │    │ • Effects        │                    │       │
│  │  └────────┬────────┘    └────────┬────────┘                    │       │
│  │           │                      │                             │       │
│  │           └──────────┬───────────┘                             │       │
│  │                      │ mpsc (Rust channels)                    │       │
│  │                      ▼                                          │       │
│  │              ┌─────────────────┐                                │       │
│  │              │ WebSocket Client│                                │       │
│  │              └────────┬────────┘                                │       │
│  └───────────────────────┼─────────────────────────────────────────┘       │
│                          │                                                   │
└──────────────────────────┼───────────────────────────────────────────────────┘
                           │
                           │ WebSocket (wss://)
                           │
                           ▼
                    ┌─────────────┐
                    │   Serveurs   │
                    └─────────────┘
```

---

## 🔧 Architecture par Composant

### 1. API Gateway (Axum)

Point d'entrée unique pour tous les clients.

**Responsabilités :**
- Authentification OAuth2 (Google/Discord)
- Génération/Validation JWT
- Routing vers les clusters de jeu
- Gestion des connexions WebSocket initiales
- API REST pour les opérations non-temps réel

```rust
// Structure simplifiée
pub struct ApiGateway {
    auth_service: Arc<AuthService>,
    cluster_router: Arc<ClusterRouter>,
    db: Arc<PgPool>,
}

// Endpoints
// - POST /auth/oauth/{provider}     → OAuth flow
// - POST /auth/refresh              → Refresh JWT
// - GET  /api/cluster/status        → Liste des clusters
// - WS   /ws/connect                → WebSocket upgrade + routing
```

### 2. Game Server Cluster

Chaque cluster gère un ensemble de zones spatiales.

**Par instance de serveur :**
- 1-5 zones spatiales (selon la densité)
- ~200-500 joueurs simultanés
- Tick rate : 30Hz (simulation) / 20Hz (network)
- State sync avec Redis (pour les cross-zone transactions)

```rust
pub struct GameServer {
    // Bevy ECS headless
    app: App,

    // Zone IDs gérés par cette instance
    zone_ids: Vec<ZoneId>,

    // Connexions WebSocket actives
    connections: Slab<PlayerConnection>,

    // Spatial Hashing pour Area of Interest
    spatial_index: SpatialHashGrid<EntityId>,

    // Redis pour cross-cluster
    redis: Arc<redis::Client>,
}
```

### 3. Spatial Hashing (Area of Interest)

Optimisation critique : ne pas envoyer tout l'état du monde à chaque joueur.

```rust
// Grille 2D pour la gestion de proximité
pub struct SpatialHashGrid<T> {
    cell_size: f32,
    grid: HashMap<(i32, i32), Vec<T>>,
}

// Un joueur ne reçoit que les updates des :
// - Même cellule
// - 8 cellules adjacentes
// - Entités avec InterestRadius qui chevauchent sa cellule
```

### 4. Client Bevy

Rendu graphique + prédiction client-side.

```rust
pub struct R2Client {
    // Bevy app
    app: App,

    // WebSocket vers le serveur
    ws: WebSocket,

    // État du monde (interpolé)
    world: World,

    // Entités du joueur local
    local_entities: HashMap<EntityId, Entity>,
}

// Systems Bevy
// - input_system         → Capture clics/clavier
// - prediction_system    → Mouvement client-side (pour le feel)
// - interpolation_system → Lissage positions autres joueurs
// - render_system        → Rendu WebGPU
// - ui_bridge_system     → Sync Bevy ↔ Dioxus
```

### 5. Dioxus UI Bridge

Communication bidirectionnelle Bevy ↔ Dioxus.

```rust
// Bevy → Dioxus (Game State → UI)
channel_game_to_ui: mpsc::Sender<UIEvent>,

// Dioxus → Bevy (UI Actions → Game)
channel_ui_to_game: mpsc::sender<UIAction>,
```

---

## 🔄 Flux de Données

### Mouvement Joueur

```
┌─────────┐         ┌──────────┐         ┌──────────┐
│ Client  │         │  Serveur │         │   DB     │
└────┬────┘         └─────┬────┘         └────┬─────┘
     │                   │                   │
     │ Clic (x,y)        │                   │
     ├──────────────────►│                   │
     │  MoveTo           │                   │
     │                   │                   │
     │                   │ Vérification      │
     │                   │ (cooldown, map)   │
     │                   │                   │
     │                   │ Update ECS        │
     │                   │ Position +=       │
     │                   │                   │
     │ WorldUpdate       │                   │
     │◄──────────────────┤                   │
     │ (nouvelle pos)    │                   │
     │                   │                   │
     │ Interpolation     │                   │
     │ (smooth)          │                   │
     │                   │                   │
     │ Render            │                   │
     │                   │                   │
```

### Combat PvP

```
┌─────────┐         ┌──────────┐         ┌──────────┐
│ Client  │         │  Serveur │         │   DB     │
└────┬────┘         └─────┬────┘         └────┬─────┘
     │                   │                   │
     │ UseSkill          │                   │
     ├──────────────────►│                   │
     │                   │                   │
     │                   │ Validation (range,│
     │                   │ cooldown, energy) │
     │                   │                   │
     │                   │ Calcul dégâts     │
     │                   │                   │
     │                   │ Update HP cible   │
     │                   ├──────────────────►│
     │                   │ Log combat        │
     │ CombatEvent       │                   │
     │◄──────────────────┤                   │
     │ (damage, vfx)     │                   │
     │                   │                   │
     │ VFX (particules,  │                   │
     │      sound)       │                   │
     │                   │                   │
```

### Transaction Marché

```
┌─────────┐         ┌──────────┐         ┌──────────┐         ┌─────────┐
│ Client  │         │  Serveur │         │  Redis   │         │   DB    │
└────┬────┘         └─────┬────┘         └────┬─────┘         └────┬────┘
     │                   │                   │                   │
     │ BuyOrder          │                   │                   │
     ├──────────────────►│                   │                   │
     │                   │                   │                   │
     │                   │ Check market      │                   │
     │                   ├──────────────────►│                   │
     │                   │ (lock item)       │                   │
     │                   │                   │                   │
     │                   │ Vérif fonds       │                   │
     │                   ├───────────────────┼──────────────────►│
     │                   │ (player balance)  │                   │
     │                   │                   │                   │
     │                   │ Execute tx        │                   │
     │                   ├───────────────────┼──────────────────►│
     │                   │                   │                   │
     │ MarketUpdate      │                   │                   │
     │◄──────────────────┤                   │                   │
     │                   │                   │                   │
```

---

## 📊 Messages Réseau

Voir [`02-networking.md`](./02-networking.md) pour le protocole complet.

---

## 🚀 Scalability Strategy

### Vertical Scaling (Phase 1)
- 1 gros serveur par zone
- 200-500 joueurs max

### Horizontal Scaling (Phase 2)
- Plusieurs instances par zone (load balancing)
- Redis shared state

### Geographic Distribution (Phase 3)
- Clusters par région (EU, US, ASIA)
- Cross-cluster travel via Jump Gates

---

## 🔒 Sécurité

- **Serveur Authorité** : Le client ne fait que des suggestions
- **Validation serveur** : Toutes les actions sont vérifiées
- **Rate limiting** : Anti-spam, anti-DDoS
- **Anti-cheat** : Détection de Mouvement impossible, speed hacks

---

*Architecture v0.1.0 - Sujet à évolution*
