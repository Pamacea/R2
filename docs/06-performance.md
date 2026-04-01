# Performance R2 - Optimisations & Scaling

> **Version** : 0.1.0 | **Dernière mise à jour** : 2025-04-01

---

## ⚡ Objectifs de Performance

| Métrique | Cible | Comment mesurer |
|----------|-------|-----------------|
| Tick Rate Serveur | 30 Hz | Simulation loop |
| Network Update Rate | 20 Hz | WorldUpdate messages |
| Input Latency | < 50ms | Ping client ↔ serveur |
| Frame Rate Client | 60 FPS | Bevy renderer |
| Memory par joueur | < 100 Mo | Profiling Rust |
| Bandwidth par joueur | < 15 KB/s | Network monitoring |

---

## 🖥️ Optimisations Serveur

### 1. ECS Parallel Systems

```rust
// Les systèmes indépendants tournent en parallèle
app.add_systems(
    Update,
    (
        movement_system,
        regen_system,
        cooldown_system,
        ai_think_system, // CPU intensive
    )
    .into_iter() // Chaque system dans son thread
);

// Pour les systèmes qui partagent des données
app.add_systems(
    Update,
    (
        combat_system.before(damage_system),
        damage_system.after(combat_system),
    ).chain(), // Séquentiel
);
```

### 2. Spatial Hashing

```rust
// r2_server/src/spatial/hash.rs

pub struct SpatialHashGrid<T> {
    cell_size: f32,
    grid: HashMap<(i32, i32), Vec<T>>,
    world_size: (f32, f32),
}

impl<T: Copy + Eq + Hash> SpatialHashGrid<T> {
    pub fn new(cell_size: f32, world_size: (f32, f32)) -> Self {
        Self {
            cell_size,
            grid: HashMap::with_capacity(1024),
            world_size,
        }
    }

    #[inline]
    fn cell_coords(&self, x: f32, y: f32) -> (i32, i32) {
        (
            (x / self.cell_size).floor() as i32,
            (y / self.cell_size).floor() as i32,
        )
    }

    pub fn insert(&mut self, entity_id: T, x: f32, y: f32) {
        let cell = self.cell_coords(x, y);
        self.grid.entry(cell).or_default().push(entity_id);
    }

    pub fn query_radius(&self, x: f32, y: f32, radius: f32) -> Vec<T> {
        let cell = self.cell_coords(x, y);
        let cell_radius = (radius / self.cell_size).ceil() as i32;

        let mut result = Vec::with_capacity(32);

        for dx in -cell_radius..=cell_radius {
            for dy in -cell_radius..=cell_radius {
                if let Some(entities) = self.grid.get(&(cell.0 + dx, cell.1 + dy)) {
                    result.extend(*entities);
                }
            }
        }

        result
    }

    /// Optimisation : Itérer uniquement sur les cellules occupées
    pub fn occupied_cells(&self) -> impl Iterator<Item = &(i32, i32)> {
        self.grid.keys()
    }
}
```

### 3. Interest Management (AOI)

```rust
// Seulement envoyer ce que le joueur doit voir
pub struct AOIConfig {
    pub vision_radius: f32,      // Rayon de vision
    pub grid_cell_size: f32,     // Taille cellule spatiale
}

pub fn get_relevant_entities(
    player_pos: Vec2,
    world: &World,
    spatial: &SpatialHashGrid<EntityId>,
    config: &AOIConfig,
) -> Vec<EntitySnapshot> {
    // 1. Récupérer entités dans la zone
    let nearby = spatial.query_radius(player_pos.x, player_pos.y, config.vision_radius);

    // 2. Filtrer par occlusion (optionnel)
    // ...

    // 3. Sérialiser uniquement les composants nécessaires
    nearby
        .iter()
        .filter_map(|&id| world.get_snapshot(id))
        .collect()
}
```

### 4. Object Pooling

```rust
// Réutiliser les allocations pour éviter GC
pub struct EntityPool {
    free: Vec<EntityId>,
    next_id: u64,
}

impl EntityPool {
    pub fn acquire(&mut self) -> EntityId {
        self.free.pop().unwrap_or_else(|| {
            let id = self.next_id;
            self.next_id += 1;
            EntityId(id)
        })
    }

    pub fn release(&mut self, id: EntityId) {
        self.free.push(id);
    }
}
```

### 5. Batch Network Messages

```rust
// Au lieu d'envoyer un message par update, batcher
pub struct NetworkBatch {
    buffer: Vec<R2Message>,
    last_send: Instant,
    send_interval: Duration, // 50ms = 20Hz
}

impl NetworkBatch {
    pub fn add(&mut self, msg: R2Message) {
        self.buffer.push(msg);
    }

    pub fn flush(&mut self) -> Option<Vec<R2Message>> {
        if self.buffer.is_empty() {
            return None;
        }

        if self.last_send.elapsed() < self.send_interval {
            return None;
        }

        self.last_send = Instant::now();
        Some(std::mem::take(&mut self.buffer))
    }
}
```

---

## 🎨 Optimisations Client (Bevy)

### 1. Frustum Culling

```rust
// Ne pas rendre ce qui est hors caméra
#[derive(Component)]
pub struct VisibleInCamera;

pub fn frustum_culling_system(
    mut commands: Commands,
    query: Query<(Entity, &GlobalTransform, &Aabb), Without<VisibleInCamera>>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    let (camera, cam_transform) = camera.single();
    let frustum = camera.frustum(cam_transform);

    for (entity, transform, aabb) in query.iter() {
        if frustum.intersects_aabb(aabb, transform) {
            commands.entity(entity).insert(VisibleInCamera);
        } else {
            commands.entity(entity).remove::<VisibleInCamera>();
        }
    }
}

// Render system ne filtre que les visibles
fn render_system(query: Query<&Mesh, With<VisibleInCamera>>) {
    // ...
}
```

### 2. LOD (Level of Detail)

```rust
#[derive(Component)]
pub struct LOD {
    levels: Vec<Handle<Mesh>>,
}

pub fn lod_system(
    mut commands: Commands,
    query: Query<(Entity, &GlobalTransform, &LOD)>,
    camera: Query<&GlobalTransform, With<Camera>>,
) {
    let cam_pos = camera.single().translation();

    for (entity, transform, lod) in query.iter() {
        let distance = transform.translation.distance(cam_pos);

        let lod_index = match distance {
            d if d < 100.0 => 0,  // High detail
            d if d < 500.0 => 1,  // Medium
            _ => 2,               // Low
        };

        commands.entity(entity).insert(lod.levels[lod_index].clone());
    }
}
```

### 3. Instanced Rendering

```rust
// Pour les objets répétitifs (astéroïdes, projectiles)
#[derive(Component)]
pub struct Instance {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

fn instanced_render_system(
    query: Query<&Instance>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Un seul draw call pour milliers d'instances
    // GPU fait la transformation
}
```

### 4. Interpolation Smooth

```rust
#[derive(Component)]
pub struct InterpolatedPosition {
    pub current: Vec3,
    pub target: Vec3,
    pub alpha: f32, // 0-1
}

pub fn interpolate_positions(
    mut query: Query<&mut InterpolatedPosition>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    let lerp_speed = 10.0; // Ajuster selon latence

    for mut interp in query.iter_mut() {
        interp.alpha = (interp.alpha + lerp_speed * dt).min(1.0);

        interp.current = interp.current.lerp(interp.target, interp.alpha);
    }
}
```

### 5. CPU-Side Particles

```rust
// Particules simples côté CPU pour économiser GPU
#[derive(Component)]
pub struct CpuParticles {
    particles: Vec<Particle>,
    count: usize,
}

impl CpuParticles {
    pub fn spawn(&mut self, position: Vec3, velocity: Vec3, lifetime: f32) {
        if self.particles.len() < self.count {
            self.particles.push(Particle {
                position,
                velocity,
                lifetime,
                age: 0.0,
            });
        }
    }
}
```

---

## 📡 Optimisations Réseau

### 1. Delta Compression

```rust
// N'envoyer que les changements
pub struct EntityDelta {
    pub id: EntityId,
    pub position_delta: Option<Vec3>,
    pub rotation_delta: Option<Quat>,
    pub stats_delta: Option<StatsDelta>,
}

pub fn compute_delta(
    old: &EntitySnapshot,
    new: &EntitySnapshot,
) -> EntityDelta {
    EntityDelta {
        id: new.id,
        position_delta: if old.position != new.position {
            Some(new.position - old.position)
        } else {
            None
        },
        // ...
    }
}
```

### 2. Quantification

```rust
// Arrondir les positions pour économiser des bits
pub fn quantize_position(pos: Vec3, precision: f32) -> (i16, i16, i16) {
    (
        (pos.x / precision).round() as i16,
        (pos.y / precision).round() as i16,
        (pos.z / precision).round() as i16,
    );
}

pub fn unquantize_position(quantized: (i16, i16, i16), precision: f32) -> Vec3 {
    Vec3::new(
        quantized.0 as f32 * precision,
        quantized.1 as f32 * precision,
        quantized.2 as f32 * precision,
    );
}
```

### 3. Priority Queue

```rust
// Les messages importants passent en priorité
pub enum NetworkPriority {
    High,    // Combat, mort
    Medium,  // Mouvement
    Low,     // Chat, emotes
}

pub struct PrioritizedMessage {
    priority: NetworkPriority,
    msg: R2Message,
}

pub struct NetworkQueue {
    high: VecDeque<R2Message>,
    medium: VecDeque<R2Message>,
    low: VecDeque<R2Message>,
}

impl NetworkQueue {
    pub fn drain_to_bytes(&mut self, max_bytes: usize) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut remaining = max_bytes;

        // High priority
        while let Some(msg) = self.high.pop_front() {
            let msg_bytes = serialize(&msg);
            if msg_bytes.len() > remaining {
                self.high.push_front(msg);
                break;
            }
            remaining -= msg_bytes.len();
            bytes.extend_from_slice(&msg_bytes);
        }

        // Medium priority
        while let Some(msg) = self.medium.pop_front() {
            let msg_bytes = serialize(&msg);
            if msg_bytes.len() > remaining {
                self.medium.push_front(msg);
                break;
            }
            remaining -= msg_bytes.len();
            bytes.extend_from_slice(&msg_bytes);
        }

        bytes
    }
}
```

---

## 🗄️ Optimisations Database

### 1. Connection Pool

```rust
// sqlx pool configuré
let pool = sqlx::PgPoolOptions::new()
    .max_connections(20) // Selon charge
    .min_connections(5)
    .acquire_timeout(Duration::from_secs(3))
    .connect(&env::var("DATABASE_URL")?)
    .await?;
```

### 2. Prepared Statements

```rust
// Réutiliser les queries
static GET_PLAYER: Lazy<sqlx::query::Map<'static, PgPool, Player>> = Lazy::new(|| {
    sqlx::query_as!("SELECT * FROM players WHERE player_id = $1")
});

// Utilisation
let player = GET_PLAYER.fetch_one(&pool).await?;
```

### 3. Batch Inserts

```rust
// Au lieu de N inserts
// INSERT INTO combat_log ... x 100

// Faire un seul batch
sqlx::query!(
    r#"
    INSERT INTO combat_log (attacker_id, defender_id, attacker_won, damage_dealt, zone_id)
    SELECT
        unnest($1::bigint[]),
        unnest($2::bigint[]),
        unnest($3::boolean[]),
        unnest($4::bigint[]),
        $5::smallint
    "#,
    &[1, 2, 3] as &[i64],
    &[10, 20, 30] as &[i64],
    &[true, false, true] as &[bool],
    &[100, 50, 200] as &[i64],
    1i16
)
.execute(&pool)
.await?;
```

### 4. Redis Cache

```rust
// Cache les données chaudes
pub struct CacheService {
    redis: redis::Client,
}

impl CacheService {
    pub async fn get_player_stats(&self, player_id: i64) -> Option<Stats> {
        let key = format!("player:{}:stats", player_id);

        // Try cache
        if let Ok(cached) = self.redis.get(&key).await {
            if let Ok(stats) = serde_json::from_str::<Stats>(&cached) {
                return Some(stats);
            }
        }

        // Fallback DB
        // ...

        // Set cache
        self.redis
            .set_ex(&key, serde_json::to_string(&stats).unwrap(), 3600)
            .await
            .ok();
    }
}
```

---

## 📊 Profiling Tools

### Flamegraph

```bash
# Installer flamegraph
cargo install flamegraph

# Générer
cargo flamegraph --bin r2_server

# Analyser
flamegraph r2_server-flamegraph.svg
```

### Tracy Profiler

```rust
// Cargo.toml
[dependencies]
tracing = "0.1"
tracing-tracy = "0.11"

// Code
use tracing::{span, Level};

fn movement_system() {
    let _span = span!(Level::TRACE, "movement_system").entered();

    // ...
}
```

### Criterion Benchmarks

```rust
// benches/spatial_hash.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_spatial_insert(c: &mut Criterion) {
    let mut grid = SpatialHashGrid::new(100.0, (10000.0, 10000.0));

    c.bench_function("spatial_insert", |b| {
        b.iter(|| {
            grid.insert(black_box(EntityId(123)), black_box(500.0), black_box(500.0))
        })
    });
}

criterion_group!(benches, bench_spatial_insert);
criterion_main!(benches);
```

---

## 🎯 Performance Checklist

### Serveur

- [ ] ECS systems parallélisés
- [ ] Spatial hashing actif
- [ ] Interest management activé
- [ ] Network batching (20Hz)
- [ ] Object pooling
- [ ] Connection pool DB optimisé

### Client

- [ ] Frustum culling actif
- [ ] LOD implémenté
- [ ] Interpolation smooth
- [ ] Instanced rendering
- [ ] CPU particles pour effets simples

### Réseau

- [ ] Delta compression
- [ ] Position quantification
- [ ] Priority queue pour messages
- [ ] Compression zstd active

---

## 📈 Scaling Strategy

### Vertical (Phase 1)

```
1 Instance = 200-500 joueurs
CPU: 8-16 cores
RAM: 32 Go
```

### Horizontal (Phase 2)

```
Load Balancer → N instances
+ Redis shared state
+ DB partitionnée
```

### Geographic (Phase 3)

```
Cluster EU (Paris)
Cluster US (East Coast)
Cluster ASIA (Singapore)

Cross-cluster via Jump Gates
```

---

*Performance v0.1.0 - Optimisations continues*
