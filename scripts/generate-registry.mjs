/**
 * generate-registry.mjs
 * Parses sim-registry.txt and regenerates:
 *   packages/jett-api/src/constants/vars.ts
 *   packages/jett-api/src/constants/events.ts
 *   packages/jett-api/src/constants/units.ts
 *
 * Run from workspace root: node scripts/generate-registry.mjs
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, '..');

// ── Read & split the registry file ─────────────────────────────────────────

const raw = fs.readFileSync(path.join(root, 'sim-registry.txt'), 'utf-8');
const lines = raw.split(/\r?\n/);

let eventsIdx = lines.findIndex(l => l.trim() === '## EVENTS');
let unitsIdx  = lines.findIndex(l => l.trim() === '## UNITS');

if (eventsIdx === -1) { console.error('Could not find ## EVENTS section'); process.exit(1); }
if (unitsIdx  === -1) { console.error('Could not find ## UNITS section');  process.exit(1); }

const simvarLines = lines.slice(0, eventsIdx);
const eventLines  = lines.slice(eventsIdx + 1, unitsIdx);
const unitLines   = lines.slice(unitsIdx + 1);

// ── Helpers ─────────────────────────────────────────────────────────────────

/** Turn a raw SimConnect name into a UPPER_SNAKE_CASE TS identifier */
function toKey(name) {
  return name
    .toUpperCase()
    .replace(/\s+/g, '_')       // spaces → underscore
    .replace(/[:/\-\.\(\)]/g, '_') // special chars → underscore
    .replace(/_+/g, '_')        // collapse runs
    .replace(/^_+|_+$/g, '');   // strip leading/trailing
}

/** Build an array of { key, value } deduplicated by key */
function buildEntries(lines, valueTransform = v => v) {
  const entries = [];
  const seen = new Set();

  for (let raw of lines) {
    const line = raw.trim();
    if (!line || line.startsWith('#') || line.startsWith('//')) continue;

    const value = valueTransform(line);
    if (!value) continue;

    let key = toKey(value);
    if (!key || /^\d/.test(key)) key = '_' + key; // TS identifiers can't start with digit

    // Deduplicate
    let finalKey = key;
    let n = 2;
    while (seen.has(finalKey)) finalKey = `${key}_${n++}`;
    seen.add(finalKey);

    entries.push({ key: finalKey, value });
  }

  return entries;
}

// ── Units: take only the FIRST alias on each comma-separated line ────────────

function buildUnitEntries(lines) {
  const entries = [];
  const seen = new Set();

  for (let raw of lines) {
    const line = raw.trim();
    if (!line || line.startsWith('#')) continue;

    // Some lines are "XYZ" structs or metadata – skip non-unit looking lines
    const primary = line.split(',')[0].trim();
    if (!primary) continue;

    let key = toKey(primary);
    if (!key) continue;
    if (/^\d/.test(key)) key = '_' + key;

    let finalKey = key;
    let n = 2;
    while (seen.has(finalKey)) finalKey = `${key}_${n++}`;
    seen.add(finalKey);

    entries.push({ key: finalKey, value: primary });
  }

  return entries;
}

// ── Generate TypeScript source ───────────────────────────────────────────────

function renderTs(typeName, entries, header = '') {
  const lines = [`// Auto-generated from sim-registry.txt – do not edit manually`, ``];
  if (header) lines.push(header, ``);
  lines.push(`export const ${typeName} = {`);
  for (const { key, value } of entries) {
    lines.push(`  ${key}: '${value.replace(/\\/g, '\\\\').replace(/'/g, "\\'")}',`);
  }
  lines.push(`} as const;`, ``);
  lines.push(`export type ${typeName.charAt(0).toUpperCase() + typeName.slice(1)}Key = typeof ${typeName}[keyof typeof ${typeName}];`, ``);
  return lines.join('\n');
}

// ── Process ─────────────────────────────────────────────────────────────────

const simvars = buildEntries(simvarLines);
const events  = buildEntries(eventLines);
const units   = buildUnitEntries(unitLines);

console.log(`SimVars : ${simvars.length}`);
console.log(`Events  : ${events.length}`);
console.log(`Units   : ${units.length}`);

// ── Write files ──────────────────────────────────────────────────────────────

const constantsDir = path.join(root, 'packages', 'jett-api', 'src', 'constants');
fs.mkdirSync(constantsDir, { recursive: true });

fs.writeFileSync(path.join(constantsDir, 'vars.ts'),   renderTs('vars',   simvars));
fs.writeFileSync(path.join(constantsDir, 'events.ts'), renderTs('events', events));
fs.writeFileSync(path.join(constantsDir, 'units.ts'),  renderTs('units',  units));

console.log(`\nWrote to packages/jett-api/src/constants/`);
