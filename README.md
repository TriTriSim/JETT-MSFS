# JETT-MSFS

JETT-MSFS (JavaScript Environment for Tauri Tools) includes a React + TypeScript client for writing and running scripts in JETT Studio.

---

## Client-side quick start

### Prerequisites

- Node.js (LTS recommended)
- npm

### Install dependencies

```bash
cd /home/runner/work/JETT-MSFS/JETT-MSFS
npm install
```

### Run the client in development

```bash
npm run dev --workspace apps/jett-studio
```

### Build the client

```bash
npm run build --workspace apps/jett-studio
```

## Client-side structure

Main client files:

- `apps/jett-studio/src/App.tsx` — main UI and editor runtime
- `apps/jett-studio/src/main.tsx` — app bootstrap
- `apps/jett-studio/src/App.css` — styling
