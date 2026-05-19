// JETT Studio — basic connectivity & variable test
// Load this file via the "Load Script" button after clicking Connect.

(async function () {
  console.log("=== JETT test-basic.js starting ===");

  // ── 1. One-shot reads ────────────────────────────────────────────────────
  console.log("Requesting one-shot variables...");

  const altitude = await sim.getVariable(vars.ALTITUDE, units.FEET);
  console.log("Altitude (ft):", altitude ?? "no data");

  const airspeed = await sim.getVariable(vars.AIRSPEED_INDICATED, units.KNOTS);
  console.log("Airspeed (kts):", airspeed ?? "no data");

  const heading = await sim.getVariable(vars.HEADING_MAGNETIC, units.DEGREES);
  console.log("Heading (°):", heading ?? "no data");

  // ── 2. Continuous subscription @ 1 Hz ───────────────────────────────────
  console.log("Subscribing to ALTITUDE @ 1 Hz...");
  let altCount = 0;

  await sim.subscribeVariable(vars.ALTITUDE, units.FEET, 1, (value) => {
    altCount++;
    console.log(`[${altCount}] ALT: ${value.toFixed(0)} ft`);
    if (altCount >= 5) {
      sim.unsubscribeVariable(vars.ALTITUDE);
      console.log("Unsubscribed from ALTITUDE after 5 updates.");
    }
  });

  // ── 3. Event subscription ────────────────────────────────────────────────
  console.log("Subscribing to PAUSE_SET event...");
  await sim.subscribeEvent("PAUSE_SET", (data) => {
    console.log("PAUSE_SET received, data:", data);
  });

  // ── 4. Transmit an event ─────────────────────────────────────────────────
  // Toggle autopilot master on — comment out if you don't want this.
  // console.log("Transmitting AP_MASTER...");
  // await sim.transmitEvent(events.AP_MASTER);

  console.log("=== test-basic.js loaded. Waiting for live data... ===");
})();
