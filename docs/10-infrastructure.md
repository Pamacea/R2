# Infrastructure R2 - Déploiement

> **Version** : 0.1.0 | **Stratégie** : 100% Gratuit permanent
> **Hosting** : Render.com (Backend + Frontend + DB + Redis)
> **Objectif** : MMO capable de 10k joueurs (code), beta avec centaines de joueurs

---

## 🏗️ Architecture Cible

```
┌─────────────────────────────────────────────────────────────────┐
│                    R2 INFRASTRUCTURE                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   PHASE 1: 100% GRATUIT (Permanent)                            │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │ Frontend (Wasm)    │ Render Static Site│     $0/mo      │   │
│   │ Backend (Axum)     │ Render Web Service│     $0/mo      │   │
│   │ PostgreSQL         │ Render Database    │     $0/mo      │   │
│   │ Redis              │ Render Redis       │     $0/mo      │   │
│   └─────────────────────────────────────────────────────────┘   │
│   TOTAL: 100% GRATUIT (pas de trial, pas d'expiration)         │
│                                                                 │
│   LIMITS:                                                        │
│   - Web Service: 750h/mo, sleep après 15min inactivité         │
│   - Capacité estimée: 200-500 joueurs simultanés              │
│                                                                 │
│   PHASE 2: BETA (Dépassement limits)                           │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │ + Render Starter ($7/mo)              │    ~$7-15/mo     │   │
│   │ + Plus d'instances si besoin          │                  │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│   PHASE 3: PRODUCTION (10k+ joueurs)                           │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │ + Render Pro/Custom                   │    $50-200/mo    │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 📦 Services & Limits (Free Tier)

| Service | Usage | Free Tier Limits |
|---------|-------|------------------|
| **Render Web Service** | Backend | **750h/mo**, sleep après 15min inactivité |
| **Render Static Site** | Frontend | Illimité bandwidth |
| **PostgreSQL** | Database | 90 jours rétention, 1GB storage |
| **Redis** | Cache | 25MB storage |

**Capacité estimée free tier :** 200-500 joueurs simultanés

---

## ⚠️ Pourquoi Render.com ?

| Avantage | Détail |
|----------|--------|
| ✅ **VRAI gratuit** | Pas de trial, pas d'expiration, free tier permanent |
| ✅ **Backend + Frontend** | Un seul service pour tout (comme Railway) |
| ✅ **Évolutif** | Scale automatiquement si pic de joueurs |
| ✅ **WebSocket** | Supporte HTTP/2 pour connexions persistantes |
| ✅ **Rust supporté** | Containers Docker standards |
| ✅ **GitHub Auto-Deploy** | Push = Deploy automatique |
| ✅ **Monitoring** | Logs, métriques inclus |
| ✅ **HTTPS** | Certificats SSL automatiques |

---

## 🚀 Setup Render.com

### ÉTAPE 1 : Créer un compte

1. Aller sur : https://render.com
2. **Sign Up** avec GitHub (recommandé)
3. Autoriser Render à accéder à tes repos

### ÉTAPE 2 : New Blueprint

1. Cliquer **"New +"** → **"Blueprint"**
2. Sélectionner le repo : `Pamacea/R2`
3. Render détecte automatiquement `render.yaml`
4. **"Approve"**

C'est tout ! Render déploie automatiquement :
- ✅ Backend `r2-server`
- ✅ Frontend `r2-client`
- ✅ Database `r2-db`
- ✅ Redis `r2-redis`

### ÉTAPE 3 : Attendre le déploiement

- **Premier déploiement** : ~5-10 min (build Rust)
- **Déploiements suivants** : ~2-5 min (cache)

---

## 🔧 Configuration Render

### render.yaml (déjà créé à la racine)

Le fichier `render.yaml` configure tout le déploiement automatique :

```yaml
services:
  # Backend Rust
  - type: web
    name: r2-server
    runtime: docker
    dockerfilePath: ./Dockerfile.rust
    plan: free
    envVars:
      - key: DATABASE_URL
        fromDatabase:
          name: r2-db
          property: connectionString

sites:
  # Frontend Wasm
  - type: static
    name: r2-client
    buildCommand: cd r2_client && trunk build --release
    publishPath: r2_client/dist
    plan: free

databases:
  - name: r2-db
    databaseName: r2_db
    user: r2_user
    plan: free

redis:
  - name: r2-redis
    plan: free
```

---

## 🚀 Déploiement sur Render

### Méthode Automatique (Recommandée)

Le déploiement est **automatique** à chaque push sur `main` :

```bash
git add .
git commit -m "UPDATE: R2 - vX.X.X"
git push origin main
# → Render déploie automatiquement
```

### Méthode Manuelle (si besoin)

1. Dashboard Render → r2-server
2. **"Manual Deploy"**
3. Choisir la branche

---

## 🔐 Variables d'Environnement

### Via render.yaml (déjà configuré)

Les variables sont injectées automatiquement :
- `DATABASE_URL` → depuis la DB Render
- `REDIS_URL` → depuis Redis Render
- `JWT_SECRET` → généré automatiquement
- `PORT` → 3000 (Render impose le port)

### Via Dashboard (optionnel)

1. Render Dashboard → r2-server
2. **Environment** → **Add Environment Variable**
3. Ajouter ou modifier

---

## 📊 Monitoring

### Logs temps réel

```bash
# Via Dashboard
Dashboard → r2-server → Logs → Live Logs
```

### Métriques

Disponibles sur le dashboard :
- CPU Usage
- Memory Usage
- Response Time
- Request Count
- Error Rate

---

## 🌐 Frontend sur Render

Le frontend est servi comme **Static Site** :

```yaml
sites:
  - type: static
    name: r2-client
    buildCommand: cd r2_client && trunk build --release
    publishPath: r2_client/dist
```

**URL :** https://r2-client.onrender.com

---

## 💰 Coûts Prévisionnels

| Phase | Joueurs | Coût mensuel |
|-------|---------|--------------|
| **MVP Free** | 200-500 | **$0** |
| **Beta Starter** | 500-1000 | $7-15 (pas de sleep) |
| **Production** | 1000-10000 | $50-200 (scale) |

---

## 📋 Checklist Déploiement

### Render.com
- [ ] Compte Render créé avec GitHub
- [ ] Repo `Pamacea/R2` connecté
- [ ] Blueprint créé avec `render.yaml`
- [ ] Premier déploiement réussi
- [ ] Health check `/health` accessible

### Base de données
- [ ] Database `r2-db` créée automatiquement
- [ ] Connection string injecté
- [ ] Schéma SQL à exécuter (via psql ou migration)

### Frontend
- [ ] Trunk installé localement
- [ ] Build avec `trunk build --release`
- [ ] Déployé automatiquement

---

## 🎯 Étapes Suivantes

Une fois tout setup :

1. **Tester** le health check : `curl https://r2-server.onrender.com/health`
2. **Tester** WebSocket depuis le frontend
3. **Initialiser** la DB (exécuter les schémas SQL)
4. **Inviter** des potes pour la beta : https://r2-client.onrender.com

---

## 🚨 Problèmes Communs

**"Service sleeping"** : Le backend dors après 15 min d'inactivité → Premier request le réveille (~30 sec)

**"Database connection failed"** : Attendre que la DB soit prête → Vérifier les logs Render

**"WebSocket connection failed"** : Vérifier que r2-server est running → Health check d'abord

---

*Infrastructure v0.1.0 - Render.com 100% gratuit*
