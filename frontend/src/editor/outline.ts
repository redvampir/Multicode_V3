import { syntaxTree } from "@codemirror/language";
import { EditorView } from "@codemirror/view";
import { StateEffect } from "@codemirror/state";

interface OutlineItem {
  text: string;
  from: number;
}

function collectOutline(view: EditorView): OutlineItem[] {
  const tree = syntaxTree(view.state);
  const items: OutlineItem[] = [];
  tree.iterate({
    enter: node => {
      const name = node.type.name;
      if (/Heading|Header|FunctionDeclaration|ClassDeclaration/.test(name)) {
        let line = view.state.doc.lineAt(node.from).text.trim();
        line = line.replace(/^#+\s*/, "");
        items.push({ text: line, from: node.from });
      }
    }
  });
  return items;
}

export function attachOutline(view: EditorView, container: HTMLElement) {
  container.innerHTML = "";
  const list = document.createElement("ul");
  list.style.listStyle = "none";
  list.style.margin = "0";
  list.style.padding = "0";
  container.appendChild(list);

  function render() {
    const items = collectOutline(view);
    list.innerHTML = "";
    for (const item of items) {
      const li = document.createElement("li");
      li.textContent = item.text;
      li.style.cursor = "pointer";
      li.addEventListener("click", () => {
        view.dispatch({
          selection: { anchor: item.from },
          effects: EditorView.scrollIntoView(item.from, { y: "center" })
        });
        view.focus();
      });
      list.appendChild(li);
    }
  }

  render();

  const listener = EditorView.updateListener.of(update => {
    if (update.docChanged) render();
  });

  view.dispatch({ effects: StateEffect.appendConfig.of(listener) });

  return () => {
    container.innerHTML = "";
  };
}
