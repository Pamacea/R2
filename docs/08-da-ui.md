# DA & UI R2 - Tactical Industrial

> **Version** : 0.1.0 | **Dernière mise à jour** : 2025-04-01
> **Style** : Tactical Grid / Industrial Brutalist

---

## 🎨 Vision Artistique

R2 n'est pas un jeu spatial "flou et brillant". C'est une interface de commande tactique, brute et fonctionnelle. L'UI doit ressembler à du matériel militaire/industriel robuste, pas à une application web moderne.

```
┌─────────────────────────────────────────────────────────────────┐
│                    R2 UI PHILOSOPHY                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ❌ PAS DE:                      ✅ PLUTÔT:                    │
│   - Glassmorphism                 - Opacité forte              │
│   - Arrondis                      - Angles 90° / Biseaux 45°   │
│   - Flou / Blur                   - Structure nette            │
│   - Ombres douces                 - Bordures doubles           │
│   - Gradients subtils             - Segments distincts         │
│   - Animations fluides            - Transitions hardware       │
│                                                                 │
│   REFERENCES:                                                   │
│   - DarkOrbit (UI, barres segmentées)                         │
│   - League of Legends (skill slots, bordures)                │
│   - EVE Online (fenêtres opaques, données brutes)             │
│   - Interfaces militaires fictives (Alien, Terminator)        │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🎯 Design System

### Palette de Couleurs

```css
:root {
    /* === FOND === */
    --r2-bg-primary: #0a0b0d;      /* Noir profond */
    --r2-bg-secondary: #0d1117;    /* Gris très foncé */
    --r2-bg-tertiary: #161b22;     /* Gris foncé */

    /* === BORDURES === */
    --r2-border-steel: #30363d;    /* Gris acier */
    --r2-border-dark: #000000;     /* Noir pur */
    --r2-border-accent: #f0e68c;   /* Jaune indus / Or */

    /* === ACCENTS === */
    --r2-accent-amber: #f0e68c;    /* Ambre (warnings, highlight) */
    --r2-accent-green: #3fb950;    /* Vert radar (success, allies) */
    --r2-accent-red: #f85149;      /* Rouge alerte (danger, enemies) */
    --r2-accent-blue: #58a6ff;     /* Bleu info (system, allies) */
    --r2-accent-cyan: #39c5bb;     /* Cyan (energy, shields) */

    /* === TEXT === */
    --r2-text-primary: #c9d1d9;    /* Blanc cassé */
    --r2-text-secondary: #8b949e;  /* Gris clair */
    --r2-text-muted: #6e7681;      /* Gris moyen */

    /* === STAT === */
    --r2-hp: #f85149;              /* Hull (coque) */
    --r2-shield: #58a6ff;          /* Shield (bouclier) */
    --r2-energy: #f0e68c;          /* Energy (énergie) */
}
```

### Typographie

```css
:root {
    /* Police monospace obligatoire */
    --r2-font-family: 'JetBrains Mono', 'Roboto Mono', 'SF Mono', monospace;

    /* Tailles */
    --r2-text-xs: 11px;
    --r2-text-sm: 12px;
    --r2-text-base: 14px;
    --r2-text-lg: 16px;
    --r2-text-xl: 20px;
    --r2-text-2xl: 24px;

    /* Weights */
    --r2-font-normal: 400;
    --r2-font-medium: 500;
    --r2-font-bold: 700;
}
```

**Règle typographique :**
- Tout en monospace, même les labels
- Uppercase pour les titres de sections
- Tracking léger (+0.05em) pour la lisibilité

---

## 🧩 Composants UI

### 1. Fenêtre "Blindée"

```css
.r2-window {
    /* Fond */
    background: var(--r2-bg-secondary);

    /* Bordures doubles - aspect "blindé" */
    border-left: 4px solid var(--r2-border-steel);
    border-top: 2px solid var(--r2-border-steel);
    border-right: 1px solid var(--r2-border-dark);
    border-bottom: 1px solid var(--r2-border-dark);

    /* Inset shadow pour effet "creusé" */
    box-shadow:
        inset 0 0 10px rgba(0, 0, 0, 0.8),
        0 4px 6px rgba(0, 0, 0, 0.5);

    /* Position */
    position: relative;
    padding: 0;

    /* Animation d'apparition */
    animation: r2-scan-in 0.15s ease-out;
}

/* Header de fenêtre */
.r2-window-header {
    background: linear-gradient(180deg, var(--r2-bg-tertiary) 0%, var(--r2-bg-secondary) 100%);
    border-bottom: 2px solid var(--r2-border-steel);
    padding: 8px 12px;
    font-size: var(--r2-text-sm);
    font-weight: var(--r2-font-bold);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--r2-text-primary);
}

/* Content */
.r2-window-content {
    padding: 12px;
}
```

### 2. Slot d'Action (Skill)

```css
.r2-action-slot {
    /* Dimensions */
    width: 50px;
    height: 50px;

    /* Style industriel */
    background: var(--r2-bg-tertiary);
    border: 2px solid var(--r2-border-steel);
    outline: 1px solid var(--r2-border-dark);

    /* Layout */
    display: flex;
    align-items: center;
    justify-content: center;

    /* Transitions rapides (hardware feel) */
    transition: all 0.1s ease;

    /* Curseur */
    cursor: pointer;
}

/* Hover */
.r2-action-slot:hover {
    border-color: var(--r2-text-secondary);
    box-shadow: 0 0 8px rgba(255, 255, 255, 0.1);
}

/* Active / Pressed */
.r2-action-slot:active {
    transform: scale(0.95);
    border-color: var(--r2-accent-amber);
    box-shadow: 0 0 15px rgba(240, 230, 140, 0.3);
}

/* On Cooldown */
.r2-action-slot.on-cooldown {
    opacity: 0.5;
    filter: grayscale(1);
}

/* Ready (clignotement subtil) */
.r2-action-slot.ready {
    animation: r2-pulse-ready 2s infinite;
}

/* Keybind hint */
.r2-action-keybind {
    position: absolute;
    bottom: 2px;
    right: 2px;
    font-size: 9px;
    color: var(--r2-text-muted);
    text-transform: uppercase;
}
```

### 3. Barre de Vie Segmentée

```css
.r2-stat-bar {
    /* Container */
    display: flex;
    gap: 2px;
    height: 12px;

    /* Style */
    background: var(--r2-bg-primary);
    border: 1px solid var(--r2-border-steel);
    padding: 2px;
}

/* Segment individuel */
.r2-stat-segment {
    flex: 1;
    height: 100%;
    background: var(--r2-stat-color); /* défini par variante */
    border: 1px solid rgba(0, 0, 0, 0.5);
    transition: all 0.2s ease;
}

/* Segment vide */
.r2-stat-segment.empty {
    background: var(--r2-bg-primary);
    opacity: 0.3;
}

/* Segment qui vient de perdre de la vie (clignotement) */
.r2-stat-segment.damaged {
    animation: r2-flash-damage 0.3s ease;
}

/* Variante Hull */
.r2-stat-bar--hull .r2-stat-segment {
    --r2-stat-color: var(--r2-hp);
}

/* Variante Shield */
.r2-stat-bar--shield .r2-stat-segment {
    --r2-stat-color: var(--r2-shield);
}

/* Variante Energy */
.r2-stat-bar--energy .r2-stat-segment {
    --r2-stat-color: var(--r2-energy);
}
```

### 4. Bouton Industriel

```css
.r2-button {
    /* Style brut */
    background: var(--r2-bg-tertiary);
    border: 2px solid var(--r2-border-steel);
    outline: 1px solid var(--r2-border-dark);

    /* Text */
    font-family: var(--r2-font-family);
    font-size: var(--r2-text-sm);
    font-weight: var(--r2-font-bold);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--r2-text-primary);

    /* Layout */
    padding: 8px 16px;
    cursor: pointer;

    /* Transition hardware */
    transition: all 0.1s ease;
}

/* Hover */
.r2-button:hover {
    background: var(--r2-bg-secondary);
    border-color: var(--r2-text-secondary);
}

/* Active */
.r2-button:active {
    transform: translateY(1px);
    border-color: var(--r2-accent-amber);
}

/* Primary variant */
.r2-button--primary {
    background: var(--r2-accent-blue);
    border-color: var(--r2-accent-blue);
    color: #000;
}

.r2-button--primary:hover {
    background: #79c0ff;
}

/* Danger variant */
.r2-button--danger {
    background: var(--r2-accent-red);
    border-color: var(--r2-accent-red);
    color: #000;
}
```

### 5. Input / TextField

```css
.r2-input {
    /* Style industriel */
    background: var(--r2-bg-primary);
    border: 2px solid var(--r2-border-steel);
    outline: 1px solid var(--r2-border-dark);

    /* Text */
    font-family: var(--r2-font-family);
    font-size: var(--r2-text-base);
    color: var(--r2-text-primary);

    /* Layout */
    padding: 8px 12px;

    /* Placeholder */
    ::placeholder {
        color: var(--r2-text-muted);
    }
}

/* Focus */
.r2-input:focus {
    border-color: var(--r2-accent-blue);
    box-shadow: 0 0 8px rgba(88, 166, 255, 0.2);
}
```

### 6. Liste / Table

```css
.r2-list {
    /* Container */
    background: var(--r2-bg-secondary);
    border: 1px solid var(--r2-border-steel);

    /* Scrollbar custom */
    scrollbar-width: thin;
    scrollbar-color: var(--r2-border-steel) var(--r2-bg-primary);
}

/* Item */
.r2-list-item {
    /* Layout */
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;

    /* Bordure bas */
    border-bottom: 1px solid var(--r2-border-steel);

    /* Hover */
    transition: background 0.1s ease;
}

.r2-list-item:hover {
    background: var(--r2-bg-tertiary);
}

.r2-list-item:last-child {
    border-bottom: none;
}

/* Selected */
.r2-list-item.selected {
    background: rgba(88, 166, 255, 0.15);
    border-left: 3px solid var(--r2-accent-blue);
}
```

### 7. Progress Bar (Chargement, etc.)

```css
.r2-progress {
    /* Container */
    height: 4px;
    background: var(--r2-bg-primary);
    border: 1px solid var(--r2-border-steel);
    overflow: hidden;
}

/* Fill */
.r2-progress-fill {
    height: 100%;
    background: var(--r2-accent-blue);
    transition: width 0.3s ease;

    /* Pattern striped */
    background-image: linear-gradient(
        45deg,
        rgba(255, 255, 255, 0.1) 25%,
        transparent 25%,
        transparent 50%,
        rgba(255, 255, 255, 0.1) 50%,
        rgba(255, 255, 255, 0.1) 75%,
        transparent 75%,
        transparent
    );
    background-size: 4px 4px;
}

/* Indeterminate (animation) */
.r2-progress.indeterminate .r2-progress-fill {
    width: 30%;
    animation: r2-progress-stripes 1s linear infinite;
}
```

---

## 🎬 Animations

### Scan-In (Apparition fenêtre)

```css
@keyframes r2-scan-in {
    0% {
        clip-path: inset(0 0 100% 0);
        opacity: 0;
    }
    100% {
        clip-path: inset(0 0 0 0);
        opacity: 1;
    }
}
```

### Flash Damage

```css
@keyframes r2-flash-damage {
    0%, 100% {
        background: var(--r2-stat-color);
    }
    50% {
        background: #fff;
        box-shadow: 0 0 10px #fff;
    }
}
```

### Pulse Ready

```css
@keyframes r2-pulse-ready {
    0%, 100% {
        box-shadow: 0 0 5px var(--r2-accent-amber);
    }
    50% {
        box-shadow: 0 0 15px var(--r2-accent-amber);
    }
}
```

### Progress Stripes

```css
@keyframes r2-progress-stripes {
    0% {
        transform: translateX(-100%);
    }
    100% {
        transform: translateX(400%);
    }
}
```

### Glitch (Pour erreurs/Alertes)

```css
@keyframes r2-glitch {
    0%, 100% {
        transform: translate(0);
        opacity: 1;
    }
    20% {
        transform: translate(-2px, 2px);
        opacity: 0.8;
    }
    40% {
        transform: translate(-2px, -2px);
    }
    60% {
        transform: translate(2px, 2px);
    }
    80% {
        transform: translate(2px, -2px);
        opacity: 0.9;
    }
}

.r2-glitch {
    animation: r2-glitch 0.3s ease;
}
```

---

## 🖼️ Background Grid

Le fameux "grid millimétrique" :

```css
.r2-bg-grid {
    /* Grille de points très subtile */
    background-image: radial-gradient(
        var(--r2-border-steel) 1px,
        transparent 1px
    );
    background-size: 20px 20px;
    background-position: -1px -1px;

    /* Fond de base */
    background-color: var(--r2-bg-primary);
}

/* Alternative : lignes (plus prononcé) */
.r2-bg-lines {
    background-image: linear-gradient(
        var(--r2-border-steel) 1px,
        transparent 1px
    ),
    linear-gradient(
        90deg,
        var(--r2-border-steel) 1px,
        transparent 1px
    );
    background-size: 40px 40px;
    background-color: var(--r2-bg-primary);
}
```

---

## 🎨 Icônes

### Style "Outline Glow"

Pas besoin de Blender. Des icônes SVG simples avec un filtre CSS.

```css
.r2-icon {
    /* Couleur de base */
    color: var(--r2-text-secondary);

    /* Glow effect (néon gazeux) */
    filter: drop-shadow(0 0 2px currentColor);

    /* Transition */
    transition: all 0.2s ease;
}

/* Hover - glow plus intense */
.r2-icon:hover {
    color: var(--r2-accent-blue);
    filter: drop-shadow(0 0 6px currentColor);
}

/* Active - glow max */
.r2-icon:active {
    filter: drop-shadow(0 0 10px currentColor);
}

/* Variantes */
.r2-icon--hp { color: var(--r2-hp); }
.r2-icon--shield { color: var(--r2-shield); }
.r2-icon--energy { color: var(--r2-energy); }
.r2-icon--warning { color: var(--r2-accent-amber); }
```

### Sources d'icônes recommandées

- **Heroicons** (outline) - https://heroicons.com/
- **Tabler Icons** - https://tabler-icons.io/
- **Lucide** - https://lucide.dev/

Toutes en SVG, facilement stylables.

---

## 📐 Layout Patterns

### HUD Principal

```
┌─────────────────────────────────────────────────────────────────┐
│  [LOGO]             R2 - Tactical Interface        [MINI-MAP]   │
├──────────┬──────────────────────────────────────────┬───────────┤
│          │                                           │           │
│  STATS  │           VIEWPORT / GAME WORLD          │  TARGET   │
│          │                                           │           │
│  ┌────┐ │                                           │  ┌─────┐ │
│  │HP  │ │                                           │  │INFO │ │
│  │SH  │ │                                           │  └─────┘ │
│  │EN  │ │                                           │           │
│  └────┘ │                                           │   [RADAR] │
│          │                                           │           │
│  [1][2]  │                                           │           │
│  [3][4]  │                                           │           │
│  [5][6]  │                                           │           │
│          │                                           │           │
├──────────┴──────────────────────────────────────────┴───────────┤
│  [CHAT]                                    [NOTIFICATIONS]     │
└─────────────────────────────────────────────────────────────────┘
```

### Pattern de Fenêtre Flottante

```css
.r2-float-window {
    /* Position absolue */
    position: absolute;
    z-index: 100;

    /* Style blindé */
    @extend .r2-window;

    /* Ombre portée pour profondeur */
    box-shadow:
        inset 0 0 10px rgba(0, 0, 0, 0.8),
        0 8px 32px rgba(0, 0, 0, 0.8);

    /* Draggable handle */
    &.dragging {
        cursor: move;
        opacity: 0.9;
    }
}
```

---

## 🔧 Implémentation Dioxus

### Composant Skill Slot (exemple)

```rust
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct SkillSlotProps {
    pub icon: String,
    pub keybind: String,
    pub cooldown: Option<f32>,
    pub ready: bool,
    pub on_click: EventHandler<MouseEvent>,
}

pub fn SkillSlot(props: &SkillSlotProps) -> Element {
    let on_cooldown = props.cooldown.is_some();
    let cooldown_pct = props.cooldown.unwrap_or(0.0);

    rsx! {
        div {
            class: format!(
                "r2-action-slot {} {}",
                if on_cooldown { "on-cooldown" } else { "" },
                if props.ready { "ready" } else { "" }
            ),
            onclick: move |e| props.on_click.call(e),

            // Icône
            svg {
                class: "r2-icon r2-icon--skill",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                dangerous_inner_html: "{props.icon}"
            }

            // Keybind hint
            span {
                class: "r2-action-keybind",
                "{props.keybind}"
            }

            // Cooldown overlay
            if on_cooldown {
                div {
                    class: "r2-cooldown-overlay",
                    style: "height: {cooldown_pct}%;"
                }
            }
        }
    }
}
```

### Styles Globaux Dioxus

```rust
// r2_client/src/ui/styles.rs

pub static R2_STYLES: &str = r#"
:root {
    --r2-bg-primary: #0a0b0d;
    --r2-bg-secondary: #0d1117;
    --r2-bg-tertiary: #161b22;
    --r2-border-steel: #30363d;
    --r2-border-dark: #000000;
    --r2-accent-amber: #f0e68c;
    --r2-accent-green: #3fb950;
    --r2-accent-red: #f85149;
    --r2-accent-blue: #58a6ff;
    --r2-text-primary: #c9d1d9;
    --r2-text-secondary: #8b949e;
    --r2-text-muted: #6e7681;
    --r2-font-family: 'JetBrains Mono', monospace;
}

* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

body {
    font-family: var(--r2-font-family);
    background: var(--r2-bg-primary);
    color: var(--r2-text-primary);
}

/* ... tous les autres styles ... */
"#;
```

---

## 📋 Checklist UI

Pour chaque nouveau composant UI :

- [ ] **Fond** : Opaque, jamais transparent
- [ ] **Bordures** : Doubles (steel + dark), angles droits
- [ ] **Typographie** : Monospace, uppercase pour titres
- [ ] **Animations** : Rapides (0.1-0.2s), hardware feel
- [ ] **Feedback** : Visuel immédiat (border color change, glow)
- [ ] **Accessibility** : Contraste WCAG AA minimum

---

## 🎨 Inspirations Visuelles

À regarder pour référence :

| Jeu | Ce qui nous intéresse |
|-----|----------------------|
| **DarkOrbit** | Barres segmentées, slots skills, UI industrielle |
| **Albion Online** | Bordures doubles, fenêtres blindées |
| **EVE Online** | Densité d'information, opacité |
| **League of Legends** | Skill slots, HUD layout |
| **Alien (film)** | Interfaces rétro-futuristes |
| **Terminator** | Display tactical, données brutes |

---

*DA & UI v0.1.0 - Style Tactical Industrial*
