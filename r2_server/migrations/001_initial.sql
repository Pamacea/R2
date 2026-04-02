-- R2 Database Schema
-- SQLite (dev local) / PostgreSQL (production compatible)

-- ============================================
-- PLAYERS - Comptes utilisateurs
-- ============================================
CREATE TABLE IF NOT EXISTS players (
    player_id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    email TEXT,
    oauth_provider TEXT,
    oauth_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_login TEXT,
    is_banned INTEGER NOT NULL DEFAULT 0
);

-- Index pour recherche OAuth
CREATE INDEX IF NOT EXISTS idx_players_oauth ON players(oauth_provider, oauth_id);

-- ============================================
-- CHARACTERS - Personnages jouables
-- ============================================
CREATE TABLE IF NOT EXISTS characters (
    character_id INTEGER PRIMARY KEY AUTOINCREMENT,
    player_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    level INTEGER NOT NULL DEFAULT 1,
    xp INTEGER NOT NULL DEFAULT 0,
    zone_id INTEGER NOT NULL DEFAULT 1,
    x REAL NOT NULL DEFAULT 0.0,
    y REAL NOT NULL DEFAULT 0.0,
    z REAL NOT NULL DEFAULT 0.0,
    rotation REAL NOT NULL DEFAULT 0.0,
    ship_class TEXT NOT NULL DEFAULT 'scout',
    credits INTEGER NOT NULL DEFAULT 1000,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (player_id) REFERENCES players(player_id) ON DELETE CASCADE
);

-- Index pour recherche par joueur
CREATE INDEX IF NOT EXISTS idx_characters_player ON characters(player_id);

-- ============================================
-- ITEMS - Objets du jeu
-- ============================================
CREATE TABLE IF NOT EXISTS items (
    item_id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    type TEXT NOT NULL, -- 'weapon', 'shield', 'engine', 'cargo', 'consumable'
    rarity TEXT NOT NULL DEFAULT 'common', -- 'common', 'rare', 'epic', 'legendary'
    stats TEXT, -- JSON: {"damage": 100, "speed": 1.5}
    price INTEGER NOT NULL DEFAULT 0,
    stackable INTEGER NOT NULL DEFAULT 1
);

-- ============================================
-- INVENTORIES - Inventaire des personnages
-- ============================================
CREATE TABLE IF NOT EXISTS inventories (
    character_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1,
    slot INTEGER, -- NULL pour stackable, sinon position (0-39)
    equipped INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (character_id, item_id, slot),
    FOREIGN KEY (character_id) REFERENCES characters(character_id) ON DELETE CASCADE,
    FOREIGN KEY (item_id) REFERENCES items(item_id)
);

-- Index pour recherche inventaire
CREATE INDEX IF NOT EXISTS idx_inventories_character ON inventories(character_id);

-- ============================================
-- MARKET_LISTINGS - Marché joueur
-- ============================================
CREATE TABLE IF NOT EXISTS market_listings (
    listing_id INTEGER PRIMARY KEY AUTOINCREMENT,
    character_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1,
    price_per_unit INTEGER NOT NULL,
    is_buy_order INTEGER NOT NULL DEFAULT 0, -- 0 = vente, 1 = achat
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (character_id) REFERENCES characters(character_id),
    FOREIGN KEY (item_id) REFERENCES items(item_id)
);

CREATE INDEX IF NOT EXISTS idx_market_item ON market_listings(item_id);
CREATE INDEX IF NOT EXISTS idx_market_active ON market_listings(is_buy_order) WHERE is_buy_order = 0;
