import { EditorView } from "@codemirror/view";

/**
 * Attach a simple minimap to a CodeMirror editor.
 * The minimap renders a tiny representation of the document and highlights
 * the current viewport of the editor.
 */
export function attachMinimap(view: EditorView, container: HTMLElement) {
  const canvas = document.createElement("canvas");
  canvas.style.width = "100%";
  canvas.style.height = "100%";
  canvas.style.display = "block";
  container.appendChild(canvas);

  function draw() {
    const lineHeight = 2;
    const lines = view.state.doc.lines;
    canvas.width = container.clientWidth;
    canvas.height = lines * lineHeight;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    ctx.fillStyle = "#bbb";
    for (let i = 0; i < lines; i++) {
      ctx.fillRect(0, i * lineHeight, canvas.width, 1);
    }

    const totalHeight = view.scrollDOM.scrollHeight;
    const visible = view.scrollDOM.clientHeight;
    const scrollTop = view.scrollDOM.scrollTop;
    const ratio = visible / totalHeight;
    const top = (scrollTop / totalHeight) * canvas.height;
    const height = ratio * canvas.height;

    ctx.fillStyle = "rgba(0,0,0,0.2)";
    ctx.fillRect(0, top, canvas.width, height);
  }

  view.scrollDOM.addEventListener("scroll", draw);
  draw();
}

