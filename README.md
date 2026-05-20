# JETT-MSFS

JETT-MSFS (An Intuitive Tauri Toolkit for Flight Sim Developers) is a monorepo for building tools around **Microsoft Flight Simulator SimConnect** using:

- **Tauri + Rust** for native desktop integration and SimConnect bridge
- **React + TypeScript** for the Studio UI
- A reusable **TypeScript API package** for SimConnect operations (`@jett/api`)

---

## What is in this repository

This repository is organized as a workspace monorepo:

```text
JETT-MSFS/
├─ apps/
│  └─ jett-studio/              # Desktop app (React + Tauri)
│     ├─ src/                   # Frontend UI
│     └─ src-tauri/             # Rust backend + SimConnect bridge
├─ packages/
│  └─ jett-api/                 # TypeScript API wrapper used by Studio
│     └─ src/
│        ├─ sim.ts              # High-level SimConnect methods
│        └─ constants/          # Generated vars/events/units constants
├─ scripts/
│  └─ generate-registry.mjs     # Generates constants from sim-registry.txt
├─ sim-registry.txt             # Source registry for vars/events/units
└─ package.json                 # npm workspaces root
```

---

## Current architecture

### 1) `@jett/api` (package)
The API package exposes:

- `Sim` class for connect/disconnect/read/write/subscribe calls
- `vars`, `events`, `units` constant maps
- exported types for names/keys

The implementation uses Tauri `invoke` + event listeners under the hood.

### 2) `jett-studio` (app)
The Studio app provides:

- Monaco editor to write scripts
- Runtime with globals (`sim`, `vars`, `units`, `events`)
- Live console output + connection state controls

### 3) Rust/Tauri backend
`apps/jett-studio/src-tauri/src/lib.rs` exposes command handlers:

- `sim_connect`
- `sim_disconnect`
- `sim_subscribe_variable`
- `sim_unsubscribe_variable`
- `sim_get_variable`
- `sim_subscribe_event`
- `sim_transmit_event`

These commands route to the SimConnect manager thread.

---

## Setup

### Prerequisites

- Node.js (LTS recommended)
- npm
- Rust toolchain (`rustup`, `cargo`)
- Tauri system prerequisites (depends on OS)
- Microsoft Flight Simulator + SimConnect runtime (for live integration)

### Install dependencies

From the repository root:

```bash
npm install
```

### Run Studio in development mode

```bash
npm run dev --workspace apps/jett-studio
```

To run Tauri commands:

```bash
npm run tauri --workspace apps/jett-studio
```

### Build Studio

```bash
npm run build --workspace apps/jett-studio
```

---

## Registry/constants workflow

The constants in:

- `packages/jett-api/src/constants/vars.ts`
- `packages/jett-api/src/constants/events.ts`
- `packages/jett-api/src/constants/units.ts`

are generated from `sim-registry.txt`.

Regenerate after editing registry entries:

```bash
node scripts/generate-registry.mjs
```

Do not manually edit generated constants files.
