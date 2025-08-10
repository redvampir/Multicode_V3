import { invoke } from "@tauri-apps/api/tauri";

const breakpoints = new Set();

export function toggleBreakpoint(line) {
  if (breakpoints.has(line)) {
    breakpoints.delete(line);
  } else {
    breakpoints.add(line);
  }
  renderBreakpoints();
}

function renderBreakpoints() {
  const list = document.getElementById("debug-breakpoints");
  if (!list) return;
  list.innerHTML = "";
  breakpoints.forEach((ln) => {
    const li = document.createElement("li");
    li.textContent = `Line ${ln}`;
    list.appendChild(li);
  });
}

export async function run() {
  await invoke("debug_run");
}

export async function step() {
  await invoke("debug_step");
}

export async function breakExecution() {
  await invoke("debug_break");
}

export function showVariables(vars) {
  const el = document.getElementById("debug-vars");
  if (!el) return;
  el.textContent = JSON.stringify(vars, null, 2);
}
