# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-04-01

### Added
- Configuration Docker Compose pour le développement local
- Service Redis (r2-redis) pour le cache
- SQLite pour le développement local (évite les problèmes d'encodage Windows)
- Documentation dans GUIDE_LOCAL.md

### Changed
- Backend utilise SQLite en dev, prêt pour PostgreSQL en production
- Structure du projet prête pour développement full-stack

### Infrastructure
- Redis: `localhost:6379` (Docker)
- Database: SQLite en mémoire (dev local)
- Server: `http://localhost:3000`

## [0.1.0] - 2026-03-XX

### Added
- Initial project structure
- Axum backend server
- WebSocket support
- Spatial hashing for AOI (Area of Interest)
- OAuth2 + JWT authentication stubs
- Bevy ECS integration (headless)
