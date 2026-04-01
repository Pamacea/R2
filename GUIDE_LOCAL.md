# 🎮 R2 - Jouer avec ses potes (Self-Hosted)

> **Version** : 0.1.0 | **Hosted by YOU**

---

## 🚀 Lancement (UNE SEULE COMMANDE)

```bash
# Depuis le dossier R2
docker-compose up -d
```

**C'est tout !** Tout démarre :
- ✅ PostgreSQL (port 5432)
- ✅ Redis (port 6379)
- ✅ R2 Server (port 3000)

---

## 🧪 Vérifier que ça marche

```bash
# Voir les services
docker-compose ps

# Voir les logs
docker-compose logs -f r2-server

# Health check
curl http://localhost:3000/health
```

---

## 👥 Inviter tes potes

### Option 1 : En local (même réseau)

1. **Trouve ton IP** :
   ```bash
   ipconfig
   # Cherche "IPv4 Address" → 192.168.x.x
   ```

2. **Donne-leur** :
   ```
   Backend: http://TON_IP:3000
   WebSocket: ws://TON_IP:3000/ws
   ```

### Option 2 : Via Internet (ngrok)

1. **Installer ngrok** : https://ngrok.com/download
2. **Lancer** :
   ```bash
   ngrok http 3000
   ```
3. **Donne-leur l'URL** ngrok (ex: `https://abc123.ngrok.io`)

---

## 🛑 Arrêter

```bash
docker-compose down
```

---

## 🔄 Mise à jour (après un git pull)

```bash
docker-compose down
docker-compose up -d --build
```

---

## 📊 Ports utilisés

| Port | Service |
|------|---------|
| 3000 | R2 Server |
| 5432 | PostgreSQL |
| 6379 | Redis |

---

*Simple. Efficace. À toi.*
