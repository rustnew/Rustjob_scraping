

# 📄 README.md — Scraping RustJobs.dev avec Rust

> 🚀 Un scraper performant et respectueux écrit en **Rust + Tokio + Scraper** pour extraire les offres d’emploi depuis [RustJobs.dev](https://rustjobs.dev).

---

## 🎯 Objectif

Ce projet permet de :

- 🕵️‍♂️ **Scraper** la liste des offres d’emploi Rust sur [rustjobs.dev](https://rustjobs.dev).
- 🔍 **Extraire** : titre, entreprise, lieu, tags, URL, et description complète.
- 💾 **Sauvegarder** les résultats dans un fichier `rustjobs.json`.
- 🛠 **Déboguer facilement** grâce à la sauvegarde de la page HTML (`debug_page.html`).

---

## 🧰 Prérequis

### 1. Rust installé

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

→ Vérifie avec :

```bash
rustc --version
cargo --version
```

### 2. Dépendances du projet

Ajoute ceci dans ton `Cargo.toml` :

```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
scraper = "0.18"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
```

---

## 🚀 Utilisation

### 1. Clone & Build

```bash
git clone https://github.com/tonusername/ton-depot.git
cd ton-depot
cargo build --release
```

### 2. Lancer le scraper

```bash
cargo run
```

### 3. Résultats

- ✅ `rustjobs.json` → contient toutes les offres scrapées, au format JSON.
- 🔍 `debug_page.html` → copie de la page HTML pour analyse/debug.

---

## 📁 Structure du fichier JSON de sortie

Chaque offre est structurée ainsi :

```json
{
  "title": "Senior Rust Developer",
  "company": "TechCorp Inc.",
  "location": "Remote (EU)",
  "tags": ["Rust", "Tokio", "Blockchain"],
  "url": "https://rustjobs.dev/jobs/senior-rust-developer-techcorp",
  "description": "We are looking for a senior Rust developer with 5+ years..."
}
```

---

## 🧠 Comment ça marche ?

### 1. Requête HTTP

- Utilise `reqwest` avec un `User-Agent` réaliste pour éviter les blocages.
- Récupère la page principale de [rustjobs.dev](https://rustjobs.dev).

### 2. Parsing HTML

- Utilise `scraper` pour parser le DOM.
- Teste **plusieurs sélecteurs CSS** pour s’adapter à la structure du site (même si elle change).

### 3. Extraction des jobs

- Pour chaque élément trouvé, extrait :
  - Titre, entreprise, lieu, tags → depuis la page liste.
  - Description complète → en visitant chaque page de détail (avec délai de 500ms pour être poli 🤝).

### 4. Sauvegarde

- Écrit les résultats dans `rustjobs.json` (formaté avec `serde_json::to_string_pretty`).

---

## ⚠️ Avertissements & Bonnes Pratiques

- 🤖 Ce code est **pédagogique** — vérifie le `robots.txt` du site avant usage en production.
- ⏱️ Un délai de `500ms` entre chaque requête vers les pages détaillées → pour **ne pas surcharger le serveur**.
- 🧪 Le site [rustjobs.dev](https://rustjobs.dev) peut changer de structure → les sélecteurs CSS devront être ajustés.
- 📄 Le fichier `debug_page.html` t’aide à **inspecter la structure réelle** du HTML pour ajuster les sélecteurs.

---

## 🛠 Débogage

Si le scraper ne trouve pas d’offres :

1. Ouvre `debug_page.html` dans ton navigateur.
2. Inspecte les éléments → trouve les vraies classes ou balises utilisées.
3. Modifie les sélecteurs dans le code (`selectors_to_try`, `job_selector`, etc.).

Exemple de sélecteur à ajuster :

```rust
let job_selector = Selector::parse("div.bg-white.rounded-lg.shadow").unwrap();
```

---

## 🌟 Pour aller plus loin

- ✅ Ajouter un **CLI** avec `clap` pour choisir l’URL ou le délai.
- ✅ Exporter en **CSV** ou **Markdown**.
- ✅ Ajouter un **cache** pour ne pas rescrape les mêmes offres.
- ✅ Planifier l’exécution avec `cron` ou `systemd timer`.
- ✅ Ajouter des **tests unitaires** avec des fixtures HTML.

---

## 📜 Licence

MIT — Fais-en ce que tu veux 😊

---

## 🙌 Remerciements

- [reqwest](https://docs.rs/reqwest) — Client HTTP asynchrone.
- [scraper](https://docs.rs/scraper) — Parsing HTML en Rust.
- [serde](https://serde.rs/) — Sérialisation/désérialisation.
- [tokio](https://tokio.rs/) — Runtime asynchrone.
- [RustJobs.dev](https://rustjobs.dev) — Pour lister les meilleures offres Rust 💪
