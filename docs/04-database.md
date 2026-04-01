# Database Schema R2

> **Version** : 0.1.0 | **Dernière mise à jour** : 2025-04-01

---

## 💾 Vue d'Ensemble

R2 utilise **PostgreSQL 16+** pour la persistance. Le schéma est conçu pour :
- **Scalabilité** : Partitionnement horizontal possible
- **Performance** : Index appropriés, JSONB pour flexibilité
- **Intégrité** : Contraintes FK, transactions ACID

```
┌─────────────────────────────────────────────────────────────────┐
│                      POSTGRESQL R2                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │
│  │   players   │  │ characters  │  │ inventories │            │
│  │             │  │             │  │             │            │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘            │
│         │                │                │                    │
│         └────────┬───────┴────────────────┘                    │
│                  │                                             │
│         ┌────────▼─────────┐                                   │
│         │   game_data      │                                   │
│         │                  │                                   │
│         │ ┌──────────────┐ │                                   │
│         │ │   items      │ │                                   │
│         │ │   ships      │ │                                   │
│         │ │   skills     │ │                                   │
│         │ └──────────────┘ │                                   │
│         └──────────────────┘                                   │
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │
│  │    guilds   │  │   market    │  │  combat_log │            │
│  └─────────────┘  └─────────────┘  └─────────────┘            │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 📋 Schéma Complet

### 1. Auth & Players

```sql
-- ============================================
-- PLAYERS - Comptes utilisateur
-- ============================================
CREATE TABLE players (
    player_id      BIGSERIAL PRIMARY KEY,
    username      VARCHAR(32) NOT NULL UNIQUE,
    email         VARCHAR(255),

    -- OAuth (optionnel pour login classique)
    oauth_provider VARCHAR(20),
    oauth_id      VARCHAR(255),

    -- Security
    password_hash VARCHAR(255), -- bcrypt, NULL si OAuth
    created_at    TIMESTAMPTZ DEFAULT NOW(),
    last_login    TIMESTAMPTZ,
    is_banned     BOOLEAN DEFAULT FALSE,
    ban_until    TIMESTAMPTZ,

    -- Meta
    metadata      JSONB DEFAULT '{}',

    CONSTRAINT valid_oauth CHECK (
        (oauth_provider IS NULL AND oauth_id IS NULL AND password_hash IS NOT NULL)
        OR
        (oauth_provider IS NOT NULL AND oauth_id IS NOT NULL)
    )
);

CREATE INDEX idx_players_oauth ON players(oauth_provider, oauth_id);
CREATE INDEX idx_players_banned ON players(is_banned, ban_until) WHERE ban_until IS NOT NULL;

-- ============================================
-- SESSIONS - Tokens JWT actifs
-- ============================================
CREATE TABLE sessions (
    session_id    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id     BIGINT REFERENCES players(player_id) ON DELETE CASCADE,
    refresh_token VARCHAR(255) UNIQUE NOT NULL,
    expires_at    TIMESTAMPTZ NOT NULL,
    created_at    TIMESTAMPTZ DEFAULT NOW(),
    device_info   JSONB
);

CREATE INDEX idx_sessions_refresh ON sessions(refresh_token);
CREATE INDEX idx_sessions_expiry ON sessions(expires_at);
```

### 2. Characters & Game State

```sql
-- ============================================
-- CHARACTERS - Personnages créés
-- ============================================
CREATE TABLE characters (
    character_id BIGSERIAL PRIMARY KEY,
    player_id    BIGINT NOT NULL REFERENCES players(player_id) ON DELETE CASCADE,

    -- Identity
    name         VARCHAR(32) NOT NULL UNIQUE,
    level        SMALLINT DEFAULT 1,
    xp           BIGINT DEFAULT 0,

    -- Position (dernière déconnexion)
    zone_id      SMALLINT NOT NULL,
    x            FLOAT NOT NULL,
    y            FLOAT NOT NULL,
    z            FLOAT NOT NULL,
    rotation     FLOAT NOT NULL,

    -- Ship actuel
    ship_class   VARCHAR(20) NOT NULL,
    ship_loadout JSONB NOT NULL DEFAULT '{}', -- équipement actuel

    -- Stats persistantes
    credits      BIGINT DEFAULT 0,
    skill_points INTEGER DEFAULT 0,

    -- Flags
    is_deleted   BOOLEAN DEFAULT FALSE,
    created_at   TIMESTAMPTZ DEFAULT NOW(),
    updated_at   TIMESTAMPTZ DEFAULT NOW(),

    CONSTRAINT valid_position CHECK (
        x >= -50000 AND x <= 50000 AND
        y >= -50000 AND y <= 50000 AND
        z >= -1000 AND z <= 1000
    )
);

CREATE INDEX idx_characters_player ON characters(player_id);
CREATE INDEX idx_characters_zone ON characters(zone_id) WHERE is_deleted = FALSE;
CREATE INDEX idx_characters_name ON characters(name) WHERE is_deleted = FALSE;

-- ============================================
-- INVENTORIES - Stockage des items
-- ============================================
CREATE TABLE inventories (
    character_id BIGINT PRIMARY KEY REFERENCES characters(character_id) ON DELETE CASCADE,
    items        JSONB NOT NULL DEFAULT '[]', -- Array de {item_id, quantity, quality, slots}
    capacity     SMALLINT DEFAULT 50,
    updated_at   TIMESTAMPTZ DEFAULT NOW()
);

-- Alternative : Table normalisée pour requêtage (market, etc)
CREATE TABLE inventory_items (
    item_id      BIGSERIAL PRIMARY KEY,
    character_id BIGINT REFERENCES characters(character_id) ON DELETE CASCADE,
    item_type_id INTEGER NOT NULL, -- FK vers game_data.items
    quantity     INTEGER NOT NULL DEFAULT 1,
    quality      VARCHAR(10) NOT NULL DEFAULT 'common',
    durability   FLOAT,
    max_durability FLOAT,
    slots        JSONB, -- {weapon: [1,2], utility: [3]}
    created_at   TIMESTAMPTZ DEFAULT NOW(),

    CONSTRAINT valid_quality CHECK (quality IN ('common', 'uncommon', 'rare', 'epic', 'legendary'))
);

CREATE INDEX idx_inventory_items_char ON inventory_items(character_id);
CREATE INDEX idx_inventory_items_type ON inventory_items(item_type_id);

-- ============================================
-- SKILLS - Skills débloqués
-- ============================================
CREATE TABLE character_skills (
    character_id BIGINT REFERENCES characters(character_id) ON DELETE CASCADE,
    skill_id     SMALLINT NOT NULL,
    level        SMALLINT DEFAULT 1,
    xp           INTEGER DEFAULT 0,

    PRIMARY KEY (character_id, skill_id)
);
```

### 3. Economy & Market

```sql
-- ============================================
-- MARKET_ORDERS - Ordres d'achat/vente
-- ============================================
CREATE TABLE market_orders (
    order_id     BIGSERIAL PRIMARY KEY,
    character_id BIGINT REFERENCES characters(character_id),

    order_type   VARCHAR(4) NOT NULL, -- 'BUY' ou 'SELL'
    item_type_id INTEGER NOT NULL,
    quantity     INTEGER NOT NULL,
    unit_price   BIGINT NOT NULL, -- En crédits

    -- Location
    station_id   SMALLINT NOT NULL,
    zone_id      SMALLINT NOT NULL,

    -- Status
    is_active    BOOLEAN DEFAULT TRUE,
    filled_quantity INTEGER DEFAULT 0,
    created_at   TIMESTAMPTZ DEFAULT NOW(),
    expires_at   TIMESTAMPTZ NOT NULL,

    CONSTRAINT valid_order_type CHECK (order_type IN ('BUY', 'SELL'))
    CONSTRAINT valid_price CHECK (unit_price > 0)
    CONSTRAINT not_expired CHECK (expires_at > created_at)
);

CREATE INDEX idx_market_active ON market_orders(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_market_item ON market_orders(item_type_id, is_active) WHERE is_active = TRUE;
CREATE INDEX idx_market_station ON market_orders(station_id, is_active) WHERE is_active = TRUE;
CREATE INDEX idx_market_price ON market_orders(item_type_id, unit_price) WHERE is_active = TRUE;

-- ============================================
-- MARKET_HISTORY - Historique des transactions
-- ============================================
CREATE TABLE market_history (
    history_id   BIGSERIAL PRIMARY KEY,
    item_type_id INTEGER NOT NULL,
    quantity     INTEGER NOT NULL,
    unit_price   BIGINT NOT NULL,
    station_id   SMALLINT NOT NULL,
    timestamp    TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_market_history_item ON market_history(item_type_id, timestamp DESC);
CREATE INDEX idx_market_history_time ON market_history(timestamp DESC);

-- ============================================
-- TRANSACTIONS - Log des transactions entre joueurs
-- ============================================
CREATE TABLE transactions (
    transaction_id BIGSERIAL PRIMARY KEY,
    from_char_id   BIGINT REFERENCES characters(character_id),
    to_char_id     BIGINT REFERENCES characters(character_id),
    amount         BIGINT NOT NULL,
    transaction_type VARCHAR(20) NOT NULL, -- 'trade', 'market', 'gift'
    description    TEXT,
    timestamp      TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_transactions_from ON transactions(from_char_id, timestamp DESC);
CREATE INDEX idx_transactions_to ON transactions(to_char_id, timestamp DESC);
```

### 4. Social - Guilds

```sql
-- ============================================
-- GUILDS - Alliances de joueurs
-- ============================================
CREATE TABLE guilds (
    guild_id     SERIAL PRIMARY KEY,
    name         VARCHAR(32) NOT NULL UNIQUE,
    tag          VARCHAR(5) NOT NULL UNIQUE,
    leader_id    BIGINT REFERENCES characters(character_id),

    -- Stats
    level        SMALLINT DEFAULT 1,
    xp           BIGINT DEFAULT 0,
    credits      BIGINT DEFAULT 0,

    -- Settings
    is_recruiting BOOLEAN DEFAULT TRUE,
    min_level    SMALLINT DEFAULT 1,
    description  TEXT,

    created_at   TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_guilds_recruiting ON guilds(is_recruiting) WHERE is_recruiting = TRUE;

-- ============================================
-- GUILD_MEMBERS - Membres et rangs
-- ============================================
CREATE TABLE guild_members (
    character_id BIGINT PRIMARY KEY REFERENCES characters(character_id),
    guild_id     INTEGER REFERENCES guilds(guild_id) ON DELETE CASCADE,

    rank         VARCHAR(20) NOT NULL DEFAULT 'member', -- 'leader', 'officer', 'member'
    joined_at    TIMESTAMPTZ DEFAULT NOW(),

    CONSTRAINT valid_rank CHECK (rank IN ('leader', 'officer', 'member'))
);

CREATE INDEX idx_guild_members_guild ON guild_members(guild_id);
```

### 5. Combat & Logs

```sql
-- ============================================
-- COMBAT_LOG - Historique des combats PvP
-- ============================================
CREATE TABLE combat_log (
    log_id       BIGSERIAL PRIMARY KEY,

    -- Participants
    attacker_id  BIGINT REFERENCES characters(character_id),
    defender_id  BIGINT REFERENCES characters(character_id),

    -- Résultat
    attacker_won BOOLEAN,
    damage_dealt BIGINT,

    -- Contexte
    zone_id      SMALLINT NOT NULL,
    timestamp    TIMESTAMPTZ DEFAULT NOW(),

    -- Meta (ship classes, etc)
    metadata     JSONB DEFAULT '{}'
);

CREATE INDEX idx_combat_attacker ON combat_log(attacker_id, timestamp DESC);
CREATE INDEX idx_combat_defender ON combat_log(defender_id, timestamp DESC);
CREATE INDEX idx_combat_zone ON combat_log(zone_id, timestamp DESC);
```

### 6. Game Data (Static)

```sql
-- ============================================
-- ITEMS - Définition des items
-- ============================================
CREATE TABLE items (
    item_id      SERIAL PRIMARY KEY,
    name         VARCHAR(64) NOT NULL,
    type         VARCHAR(20) NOT NULL, -- 'weapon', 'utility', 'passive', 'resource', 'module'
    subtype      VARCHAR(20),
    rarity       VARCHAR(10) NOT NULL DEFAULT 'common',

    -- Stats
    base_value   BIGINT NOT NULL DEFAULT 0, -- Prix base

    -- Effects (JSONB pour flexibilité)
    stats        JSONB NOT NULL DEFAULT '{}',
    -- Ex: {damage: 100, range: 500, energy_cost: 20}

    description  TEXT,
    icon_url     VARCHAR(255),
    is_tradeable BOOLEAN DEFAULT TRUE,

    CONSTRAINT valid_rarity CHECK (rarity IN ('common', 'uncommon', 'rare', 'epic', 'legendary'))
);

CREATE INDEX idx_items_type ON items(type, subtype);

-- ============================================
-- SHIPS - Définition des vaisseaux
-- ============================================
CREATE TABLE ships (
    ship_id      SERIAL PRIMARY KEY,
    name         VARCHAR(64) NOT NULL,
    class        VARCHAR(20) NOT NULL, -- 'fighter', 'frigate', 'cruiser', 'hauler'
    tier         SMALLINT NOT NULL, -- 1-5

    -- Stats de base
    base_hull    INTEGER NOT NULL,
    base_shield  INTEGER NOT NULL,
    base_energy  INTEGER NOT NULL,

    base_speed   FLOAT NOT NULL,
    turn_rate    FLOAT NOT NULL,

    -- Slots
    weapon_slots SMALLINT NOT NULL,
    utility_slots SMALLINT NOT NULL,
    passive_slots SMALLINT NOT NULL,

    cargo_capacity INTEGER NOT NULL,

    -- Meta
    description  TEXT,
    model_url    VARCHAR(255),
    base_value   BIGINT NOT NULL,

    CONSTRAINT valid_tier CHECK (tier BETWEEN 1 AND 5)
);

-- ============================================
-- ZONES - Définition des zones de jeu
-- ============================================
CREATE TABLE zones (
    zone_id      SMALLINT PRIMARY KEY,
    name         VARCHAR(64) NOT NULL,
    type         VARCHAR(20) NOT NULL, -- 'safe', 'pvp', 'pve', 'arena'
    level_range  VARCHAR(7), -- '1-10', '10-20'

    -- Bounds
    min_x        FLOAT,
    max_x        FLOAT,
    min_y        FLOAT,
    max_y        FLOAT,

    -- Features
    has_station  BOOLEAN DEFAULT FALSE,
    is_populated BOOLEAN DEFAULT FALSE,
    danger_level SMALLINT DEFAULT 1, -- 1-10

    CONSTRAINT valid_type CHECK (type IN ('safe', 'pvp', 'pve', 'arena'))
);
```

---

## 🔄 Procédures Stockées

### Market Match Making

```sql
-- Trouver et exécuter un match achat/vente
CREATE OR REPLACE FUNCTION match_market_order(
    p_buy_order_id BIGINT,
    p_buyer_id BIGINT
) RETURNS BOOLEAN AS $$
DECLARE
    v_sell_order RECORD;
    v_quantity INTEGER;
BEGIN
    -- Trouver le meilleur ordre de vente correspondant
    FOR v_sell_order IN
        SELECT order_id, character_id, quantity, unit_price
        FROM market_orders
        WHERE order_type = 'SELL'
        AND item_type_id = (SELECT item_type_id FROM market_orders WHERE order_id = p_buy_order_id)
        AND unit_price <= (SELECT unit_price FROM market_orders WHERE order_id = p_buy_order_id)
        AND is_active = TRUE
        ORDER BY unit_price ASC, created_at ASC
        LIMIT 1
    LOOP
        -- Calculer quantité échangée
        v_quantity := LEAST(
            (SELECT quantity FROM market_orders WHERE order_id = p_buy_order_id),
            v_sell_order.quantity
        );

        -- Transférer items et crédits
        -- ... (logique de transaction)

        -- Logger dans historique
        INSERT INTO market_history (item_type_id, quantity, unit_price, station_id)
        VALUES (
            (SELECT item_type_id FROM market_orders WHERE order_id = p_buy_order_id),
            v_quantity,
            v_sell_order.unit_price,
            (SELECT station_id FROM market_orders WHERE order_id = p_buy_order_id)
        );

        RETURN TRUE;
    END LOOP;

    RETURN FALSE;
END;
$$ LANGUAGE plpgsql;
```

---

## 📊 Partitionnement (Pour scalabilité)

```sql
-- Partitionner combat_log par mois
CREATE TABLE combat_log_2025_04 PARTITION OF combat_log
    FOR VALUES FROM ('2025-04-01') TO ('2025-05-01');

-- Partitionner market_history par trimestre
CREATE TABLE market_history_2025_q2 PARTITION OF market_history
    FOR VALUES FROM ('2025-04-01') TO ('2025-07-01');
```

---

## 🔍 Index Stratégiques

```sql
-- Pour les requêtes fréquentes

-- Top players
CREATE INDEX idx_characters_xp ON characters(xp DESC) WHERE is_deleted = FALSE;

-- Market research
CREATE INDEX idx_market_trending ON market_history(item_type_id, timestamp DESC);

-- Guild leaderboard
CREATE INDEX idx_guilds_xp ON guilds(xp DESC);
```

---

## 💡 Tips PostgreSQL

### 1. JSONB pour flexibilité

```sql
-- Requêter dans JSONB
SELECT character_id, ship_loadout->'weapons' AS weapons
FROM characters
WHERE ship_loadout->>'ship_class' = 'fighter';

-- Index sur JSONB
CREATE INDEX idx_ship_class ON characters USING GIN (ship_loadout);
```

### 2. Upsert pour market

```sql
INSERT INTO market_orders (character_id, order_type, item_type_id, ...)
VALUES (...)
ON CONFLICT (character_id, item_type_id) DO UPDATE
SET quantity = market_orders.quantity + EXCLUDED.quantity;
```

### 3. Full-text search pour guilds

```sql
ALTER TABLE guilds ADD COLUMN search_vector tsvector;

CREATE INDEX idx_guilds_search ON guilds USING GIN (search_vector);

UPDATE guilds SET search_vector = to_tsvector('french', name || ' ' || COALESCE(description, ''));
```

---

*Database Schema v0.1.0 - Évolutif selon besoins*
