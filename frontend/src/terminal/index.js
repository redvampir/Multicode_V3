import { Terminal } from "xterm";
import "xterm/css/xterm.css";
import { Command } from "@tauri-apps/api/shell";

export function attachTerminal(element) {
  const term = new Terminal();
  term.open(element);

  const shell = new Command("sh", [], { stdout: "piped", stderr: "piped" });

  shell.stdout.on("data", (line) => term.write(line));
  shell.stderr.on("data", (line) => term.write(line));
  term.onData((data) => {
    shell.write(data).catch(() => {});
  });
  shell.spawn();
}
