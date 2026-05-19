# JETT-MSFS

JETT-MSFS (JavaScript Environment for Tauri Tools) is a monorepo for building tools around **Microsoft Flight Simulator SimConnect** using:

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

From repo root:

```bash
npm install
```

### Run Studio in development mode

From repo root:

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

---

## Recommended project organization (next step)

To keep this project maintainable as it grows:

1. **Keep app/package boundaries strict**
   - `apps/*` should contain runnable products only
   - `packages/*` should contain reusable libraries only

2. **Promote shared contracts into packages**
   - Move shared types/protocol definitions into a dedicated shared package if both frontend and backend need them

3. **Define ownership by folder**
   - `apps/jett-studio/src` → UI concerns
   - `apps/jett-studio/src-tauri` → native/backend concerns
   - `packages/jett-api` → public API surface and typed constants

4. **Document generated vs hand-written code**
   - Keep generator and source-of-truth files (`scripts/`, `sim-registry.txt`) obvious
   - Mark generated files clearly (already done)

5. **Add repo-level scripts over time**
   - Add root `dev`, `build`, and `check` scripts that delegate to workspaces for a smoother contributor experience

6. **Add focused docs**
   - `docs/architecture.md` (flow: UI → API → Tauri command → SimConnect)
   - `docs/contributing.md` (workflow, coding standards, release notes)
   - `docs/registry.md` (how to add/change vars/events/units safely)

---

## Suggested near-term roadmap

- Add root scripts for unified workspace build/dev/check
- Add tests for `@jett/api` transformation and event handling logic
- Add CI for install + typecheck/build on push/PR
- Add contributor docs and architecture diagram

---

## License

Add a license file (`LICENSE`) if this project is intended for public or team-wide reuse.
