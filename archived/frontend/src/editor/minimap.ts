import { EditorView } from "@codemirror/view";
import { StateEffect } from "@codemirror/state";

/**
 * Attach a minimap to a CodeMirror editor.
 * Renders a tiny overview and keeps scroll position in sync
 * between the minimap and the main editor.
 */
export function attachMinimap(view: EditorView, container: HTMLElement) {
  container.innerHTML = "";
  const canvas = document.createElement("canvas");
  canvas.style.width = "100%";
  canvas.style.height = "100%";
  canvas.style.display = "block";
  container.appendChild(canvas);

  function draw() {
    canvas.width = container.clientWidth;
    canvas.height = container.clientHeight;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Render a simple representation of lines
    const lineHeight = canvas.height / view.state.doc.lines;
    ctx.fillStyle = "#bbb";
    for (let i = 0; i < view.state.doc.lines; i++) {
      ctx.fillRect(0, i * lineHeight, canvas.width, 1);
    }

    const totalHeight = view.scrollDOM.scrollHeight;
    const visible = view.scrollDOM.clientHeight;
    const scrollTop = view.scrollDOM.scrollTop;
    const scale = canvas.height / totalHeight;
    const top = scrollTop * scale;
    const height = visible * scale;

    ctx.fillStyle = "rgba(0,0,0,0.2)";
    ctx.fillRect(0, top, canvas.width, height);
  }

  draw();
  view.scrollDOM.addEventListener("scroll", draw);

  view.dispatch({
    effects: StateEffect.appendConfig.of(
      EditorView.updateListener.of(update => {
        if (update.docChanged || update.viewportChanged) draw();
      })
    )
  });

  let dragging = false;
  function setScroll(e: MouseEvent) {
    const rect = canvas.getBoundingClientRect();
    const y = e.clientY - rect.top;
    const ratio = y / rect.height;
    view.scrollDOM.scrollTop =
      ratio * view.scrollDOM.scrollHeight - view.scrollDOM.clientHeight / 2;
  }

  canvas.addEventListener("mousedown", e => {
    dragging = true;
    setScroll(e);
  });
  const moveHandler = (e: MouseEvent) => {
    if (dragging) setScroll(e);
  };
  const upHandler = () => {
    dragging = false;
  };
  window.addEventListener("mousemove", moveHandler);
  window.addEventListener("mouseup", upHandler);

  return () => {
    view.scrollDOM.removeEventListener("scroll", draw);
    window.removeEventListener("mousemove", moveHandler);
    window.removeEventListener("mouseup", upHandler);
  };
}
