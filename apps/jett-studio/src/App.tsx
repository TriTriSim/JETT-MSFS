import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Sim, vars, units, events } from "@jett/api";
import "./App.css";

// Extend window type for script globals
declare global {
  interface Window {
    sim: Sim;
    vars: typeof vars;
    units: typeof units;
    events: typeof events;
  }
}

function App() {
  const [status, setStatus] = useState("Disconnected");
  const [logs, setLogs] = useState<string[]>([]);
  const logsEndRef = useRef<HTMLDivElement>(null);

  const appendLog = (msg: string) => {
    setLogs((prev) => [...prev.slice(-199), msg]);
  };

  useEffect(() => {
    // Expose globals for user scripts
    const sim = new Sim();
    window.sim = sim;
    window.vars = vars;
    window.units = units;
    window.events = events;

    // Intercept console.log for the output pane
    const origLog = console.log;
    console.log = (...args: unknown[]) => {
      origLog(...args);
      appendLog(args.map((a) => String(a)).join(" "));
    };

    // Listen for sim connection events
    const unlisteners: Promise<() => void>[] = [
      listen("jett-connected", () => setStatus("Connected")),
      listen("jett-disconnected", () => setStatus("Disconnected")),
      listen<string>("jett-error", (e) => appendLog("[error] " + e.payload)),
    ];

    return () => {
      console.log = origLog;
      unlisteners.forEach((p) => p.then((fn) => fn()));
    };
  }, []);

  useEffect(() => {
    logsEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs]);

  function handleFileLoad(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (ev) => {
      const src = ev.target?.result as string;
      try {
        // eslint-disable-next-line no-new-func
        new Function(src)();
      } catch (err) {
        appendLog("[script error] " + String(err));
      }
    };
    reader.readAsText(file);
    // Reset input so the same file can be re-loaded
    e.target.value = "";
  }

  async function handleConnect() {
    try {
      await window.sim.connect();
    } catch (err) {
      appendLog("[error] " + String(err));
    }
  }

  async function handleDisconnect() {
    try {
      await window.sim.disconnect();
    } catch (err) {
      appendLog("[error] " + String(err));
    }
  }

  return (
    <main className="container">
      <h1>JETT Studio</h1>

      <div className="row">
        <span>Status: <strong>{status}</strong></span>
        <button onClick={handleConnect} disabled={status === "Connected"}>
          Connect
        </button>
        <button onClick={handleDisconnect} disabled={status === "Disconnected"}>
          Disconnect
        </button>
      </div>

      <div className="row">
        <label htmlFor="script-file">Load script (.js):</label>
        <input
          id="script-file"
          type="file"
          accept=".js"
          onChange={handleFileLoad}
        />
      </div>

      <pre className="console-output">
        {logs.join("\n")}
        <div ref={logsEndRef} />
      </pre>
    </main>
  );
}

export default App;
