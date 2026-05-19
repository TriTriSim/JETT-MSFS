import { useEffect, useRef, useState, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { Sim, vars, units, events } from "@jett/api";
import Editor, { OnMount } from "@monaco-editor/react";
import "./App.css";

declare global {
  interface Window {
    sim: Sim;
    vars: typeof vars;
    units: typeof units;
    events: typeof events;
  }
}

const STARTER = `// JETT Studio — live editor
// Ctrl+Enter or click ▶ Run to execute

(async function () {
  // One-shot reads
  // Note: SimConnect classic names use spaces, e.g. "PLANE ALTITUDE" not "PLANE_ALTITUDE"
  const alt = await sim.getVariable("PLANE ALTITUDE", units.FOOT);
  console.log("Altitude:", alt?.toFixed(0), "ft");

  const spd = await sim.getVariable(vars.AIRSPEED_INDICATED, units.KNOT);
  console.log("Airspeed:", spd?.toFixed(1), "kts");

  // Subscribe at 1 Hz — unsubscribes after 5 updates
  let n = 0;
  await sim.subscribeVariable("PLANE ALTITUDE", units.FOOT, 1, (v) => {
    console.log(\`[alt \${++n}] \${v.toFixed(0)} ft\`);
    if (n >= 5) sim.unsubscribeVariable("PLANE ALTITUDE");
  });
})();
`;

// ── Build Monaco type declarations from the live vars/units/events objects ───
function buildJettDts(
  v: Record<string, string>,
  u: Record<string, string>,
  e: Record<string, string>,
): string {
  const props = (obj: Record<string, string>) =>
    Object.entries(obj)
      .map(([k, val]) => `  ${k}: '${val.replace(/\\/g, "\\\\").replace(/'/g, "\\'")}';`)
      .join("\n");

  return `
declare class Sim {
  connect(): Promise<void>;
  disconnect(): Promise<void>;
  subscribeVariable(name: string, unit: string, fps: number, cb: (value: number) => void): Promise<void>;
  unsubscribeVariable(name: string): void;
  getVariable(name: string, unit: string): Promise<number | null>;
  subscribeEvent(eventName: string, cb: (data: number) => void): Promise<void>;
  transmitEvent(eventName: string, data?: number): Promise<void>;
}
declare const sim: Sim;
declare const vars: {
${props(v)}
};
declare const units: {
${props(u)}
};
declare const events: {
${props(e)}
};
`;
}

type LogEntry = { ts: string; text: string; kind: "log" | "error" | "info" };

function App() {
  const [connected, setConnected] = useState(false);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [autoScroll, setAutoScroll] = useState(true);
  const logsEndRef = useRef<HTMLDivElement>(null);
  const editorRef = useRef<Parameters<OnMount>[0] | null>(null);

  const appendLog = useCallback((text: string, kind: LogEntry["kind"] = "log") => {
    const ts = new Date().toLocaleTimeString("en-GB", { hour12: false });
    setLogs((prev) => [...prev.slice(-499), { ts, text, kind }]);
  }, []);

  useEffect(() => {
    if (autoScroll) logsEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs, autoScroll]);

  useEffect(() => {
    const sim = new Sim();
    window.sim = sim;
    window.vars = vars;
    window.units = units;
    window.events = events;

    const origLog = console.log;
    const origError = console.error;
    console.log = (...args: unknown[]) => {
      origLog(...args);
      appendLog(args.map(String).join(" "), "log");
    };
    console.error = (...args: unknown[]) => {
      origError(...args);
      appendLog(args.map(String).join(" "), "error");
    };

    const onUnhandledRejection = (e: PromiseRejectionEvent) => {
      appendLog(String(e.reason), "error");
      e.preventDefault();
    };
    window.addEventListener("unhandledrejection", onUnhandledRejection);

    const unsubs = [
      listen("jett-connected", () => {
        setConnected(true);
        appendLog("Connected to SimConnect.", "info");
      }),
      listen("jett-disconnected", () => {
        setConnected(false);
        appendLog("Disconnected.", "info");
      }),
      listen<string>("jett-error", (e) => appendLog(e.payload, "error")),
    ];

    return () => {
      console.log = origLog;
      console.error = origError;
      window.removeEventListener("unhandledrejection", onUnhandledRejection);
      unsubs.forEach((p) => p.then((fn) => fn()));
    };
  }, [appendLog]);

  const runScript = useCallback(() => {
    appendLog("▶ Running…", "info");
    const src = editorRef.current?.getValue() ?? "";
    if (!src.trim()) { appendLog("Editor is empty (editorRef not set).", "error"); return; }
    try {
      // eslint-disable-next-line no-new-func
      const result = new Function(src)();
      if (result instanceof Promise) {
        result.catch((err: unknown) => appendLog(String(err), "error"));
      }
    } catch (err) {
      appendLog(String(err), "error");
    }
  }, [appendLog]);

  const handleEditorMount: OnMount = (editor, monaco) => {
    editorRef.current = editor;

    // Register JETT globals for IntelliSense in JS and TS files.
    // Merge with existing options so built-in completions (console, Promise, DOM…) are preserved.
    const jettDts = buildJettDts(vars as unknown as Record<string,string>, units as unknown as Record<string,string>, events as unknown as Record<string,string>);
    const libUri = "ts:jett-globals.d.ts";
    monaco.languages.typescript.javascriptDefaults.addExtraLib(jettDts, libUri);
    monaco.languages.typescript.typescriptDefaults.addExtraLib(jettDts, libUri);

    monaco.languages.typescript.javascriptDefaults.setCompilerOptions({
      ...monaco.languages.typescript.javascriptDefaults.getCompilerOptions(),
      allowJs: true,
      checkJs: true,
    });

    editor.addCommand(
      monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter,
      runScript,
    );
    editor.focus();
  };

  async function handleConnect() {
    try { await window.sim.connect(); }
    catch (err) { appendLog(String(err), "error"); }
  }

  async function handleDisconnect() {
    try { await window.sim.disconnect(); }
    catch (err) { appendLog(String(err), "error"); }
  }

  function handleOpenFile(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (ev) => {
      editorRef.current?.setValue(ev.target?.result as string);
    };
    reader.readAsText(file);
    e.target.value = "";
  }

  const fileInputRef = useRef<HTMLInputElement>(null);

  return (
    <div className="app">
      {/* ── Toolbar ── */}
      <div className="toolbar">
        <span className="app-title">JETT Studio</span>
        <div className={`status-dot ${connected ? "connected" : ""}`} />
        <span className="status-label">{connected ? "Connected" : "Disconnected"}</span>
        <div className="toolbar-sep" />
        <button className="btn btn-primary" onClick={handleConnect} disabled={connected}>
          Connect
        </button>
        <button className="btn" onClick={handleDisconnect} disabled={!connected}>
          Disconnect
        </button>
        <div className="toolbar-sep" />
        <button className="btn btn-run" onClick={runScript} title="Ctrl+Enter">
          ▶ Run
        </button>
        <button className="btn" onClick={() => fileInputRef.current?.click()}>
          Open file…
        </button>
        <input
          ref={fileInputRef}
          type="file"
          accept=".js"
          style={{ display: "none" }}
          onChange={handleOpenFile}
        />
        <div className="toolbar-spacer" />
        <button className="btn btn-dim" onClick={() => setLogs([])}>
          Clear console
        </button>
      </div>

      {/* ── Workspace ── */}
      <div className="workspace">
        {/* Editor pane */}
        <div className="editor-pane">
          <Editor
            defaultLanguage="javascript"
            defaultValue={STARTER}
            theme="vs-dark"
            onMount={handleEditorMount}
            options={{
              fontSize: 13,
              fontFamily: "'Cascadia Code', 'Fira Code', Consolas, monospace",
              fontLigatures: true,
              minimap: { enabled: false },
              scrollBeyondLastLine: false,
              tabSize: 2,
              wordWrap: "on",
              lineNumbers: "on",
              renderWhitespace: "none",
              padding: { top: 12 },
            }}
          />
        </div>

        {/* Console pane */}
        <div className="console-pane">
          <div className="console-header">
            <span>CONSOLE</span>
            <button
              className={`btn console-pin-btn ${autoScroll ? "active" : ""}`}
              title={autoScroll ? "Auto-scroll on" : "Auto-scroll off"}
              onClick={() => {
                const next = !autoScroll;
                setAutoScroll(next);
                if (next) logsEndRef.current?.scrollIntoView({ behavior: "smooth" });
              }}
            >
              {autoScroll ? "⏷" : "⏸"}
            </button>
          </div>
          <div className="console-body">
            {logs.map((l, i) => (
              <div key={i} className={`log-line log-${l.kind}`}>
                <span className="log-ts">{l.ts}</span>
                <span className="log-text">{l.text}</span>
              </div>
            ))}
            <div ref={logsEndRef} />
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;

