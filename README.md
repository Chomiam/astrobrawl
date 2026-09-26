# ⚡ AstroBrawl - Jeu Multijoueur 2D Spatial (Dark Orbit Style)

AstroBrawl est un jeu d'action spatial multijoueur 2D en temps réel dans le navigateur, inspiré des mécaniques classiques de Dark Orbit (pilotage avec inertie, minerais scintillants, tirs lasers, bannières de combat et persistance des crédits/minerais).

---

## 🏗️ Architecture du Projet

Le projet est structuré sous forme de **Monorepo Cargo Workspace** en Rust :

```text
astrobrawl/
├── Cargo.toml                  # Configuration Cargo Workspace
├── .cargo/config.toml          # Flags linker wasm32 (--import-undefined)
├── .github/workflows/
│   └── deploy-vercel.yml       # CI/CD GitHub Actions vers Vercel
├── vercel.json                 # En-têtes MIME application/wasm & COOP/COEP
├── shared/                     # Crate Rust partagé (Client + Server)
│   ├── Cargo.toml
│   └── src/lib.rs              # Modèles (PlayerShip, Laser, Mineral), Vec2, protocoles
├── client/                     # Moteur 2D WASM (Macroquad + Trunk)
│   ├── Cargo.toml
│   ├── Trunk.toml              # Bundler Trunk & hook de compilation WASM
│   ├── index.html              # Interface Dark Orbit, HUD & Plugin WebSocket Miniquad
│   ├── mq_js_bundle.js         # Runtime JavaScript WebGL Macroquad
│   ├── vercel.json
│   └── src/main.rs             # Boucle de jeu, caméra fluide, radar, particules, rendu
└── server/                     # Serveur autoritaire temps réel (Tokio + Axum)
    ├── Cargo.toml
    └── src/
        ├── main.rs             # Routes HTTP, boucle autoritaire à 30 Hz
        ├── ws.rs               # Gestionnaire WebSocket binaire
        ├── game.rs             # Simulation physique, collisions lasers, minerais
        ├── db.rs               # Persistance SQLite (crédits, minerais, comptes)
        └── auth.rs             # OAuth GitHub & signature de tokens JWT HMAC-SHA256
```

---

## 🚀 Démarrage Rapide en Local

### 1. Prérequis
- **Rust & Cargo** : `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Cible WASM** : `rustup target add wasm32-unknown-unknown`
- **Trunk** (bundler WASM) : `cargo install --locked trunk`

### 2. Lancer le Serveur Autoritaire (Port 3000)
```bash
cargo run -p astrobrawl-server
```
Le serveur initialise automatiquement la base de données SQLite `astrobrawl.db`, active la boucle physique à 30 Hz et écoute sur `http://localhost:3000`.

### 3. Lancer le Client Web (Port 8080)
Dans un autre terminal :
```bash
cd client
trunk serve
```
Ouvrez ensuite votre navigateur sur **`http://localhost:8080`**.

---

## 🎮 Commandes en Jeu

- **Propulsion (Inertie spatiale)** : Touches `Z`, `W`, `Flèche Haut` ou `Clic Droit`
- **Orientation / Visée** : Curseur de la souris
- **Tirs Laser** : `Espace` ou `Clic Gauche`
- **Récolte des Minerais** : Rapprochez votre vaisseau d'un minerai pour l'aspirer dans votre soute
- **Réapparition** : Touche `Espace` ou `R` en cas de destruction du vaisseau

---

## 🔐 Configuration de l'Authentification GitHub OAuth

1. Rendez-vous sur GitHub : **Settings > Developer Settings > OAuth Apps > New OAuth App**
2. Remplissez les champs :
   - **Application name** : `AstroBrawl`
   - **Homepage URL** : `http://localhost:8080` (ou votre URL Vercel en production)
   - **Authorization callback URL** : `http://localhost:3000/api/auth/github/callback`
3. Créez un fichier `.env` à la racine ou exportez les variables :
   ```bash
   GITHUB_CLIENT_ID="votre_client_id"
   GITHUB_CLIENT_SECRET="votre_client_secret"
   FRONTEND_URL="http://localhost:8080"
   JWT_SECRET="une_cle_secrete_ultra_securisee"
   ```

> 💡 *Note MVP : Vous pouvez également cliquer sur « Jouer Immédiatement (Invité) » pour tester sans configurer OAuth.*

---

## 🌐 Déploiement Vercel via GitHub Actions

1. Poussez le dépôt sur GitHub :
   ```bash
   gh repo create astrobrawl --public --source=. --remote=origin --push
   ```
2. Sur Vercel, liez le projet ou récupérez vos identifiants :
   - `VERCEL_TOKEN` (créé dans Vercel Account Settings > Tokens)
   - `VERCEL_ORG_ID` (trouvé dans `.vercel/project.json` ou les paramètres de l'équipe)
   - `VERCEL_PROJECT_ID` (trouvé dans les paramètres du projet Vercel)
3. Ajoutez ces 3 secrets dans votre dépôt GitHub : **Settings > Secrets and variables > Actions**.
4. À chaque push sur la branche `main`, le workflow `.github/workflows/deploy-vercel.yml` compile le client WebAssembly via Trunk et déploie le dossier `dist/` sur Vercel.
