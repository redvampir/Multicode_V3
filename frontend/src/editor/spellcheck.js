import Typo from "typo-js";
import { syntaxTree } from "@codemirror/language";
import { RangeSetBuilder } from "@codemirror/state";
import { Decoration, EditorView, ViewPlugin } from "@codemirror/view";

let dictPromise;

async function loadDictionary() {
  if (!dictPromise) {
    dictPromise = Promise.all([
      fetch("https://cdn.jsdelivr.net/npm/typo-js@1.3.0/dictionaries/en_US/en_US.aff").then(r => r.text()),
      fetch("https://cdn.jsdelivr.net/npm/typo-js@1.3.0/dictionaries/en_US/en_US.dic").then(r => r.text())
    ]).then(([aff, dic]) => new Typo("en_US", aff, dic));
  }
  return dictPromise;
}

export async function spellcheck() {
  const dictionary = await loadDictionary();

  const plugin = ViewPlugin.fromClass(class {
    constructor(view) {
      this.decorations = this.buildDeco(view);
    }
    update(update) {
      if (update.docChanged || update.viewportChanged) {
        this.decorations = this.buildDeco(update.view);
      }
    }
    buildDeco(view) {
      const builder = new RangeSetBuilder();
      const { from, to } = view.viewport;
      const tree = syntaxTree(view.state);
      tree.iterate({
        from,
        to,
        enter: node => {
          if (!node.type.is("comment")) return;
          const text = view.state.doc.sliceString(node.from, node.to);
          const word = /\b[A-Za-z]+\b/g;
          let m;
          while ((m = word.exec(text)) !== null) {
            const w = m[0];
            if (!dictionary.check(w)) {
              const start = node.from + m.index;
              const end = start + w.length;
              builder.add(start, end, Decoration.mark({ class: "cm-spell-error" }));
            }
          }
        }
      });
      return builder.finish();
    }
  }, {
    decorations: v => v.decorations
  });

  const theme = EditorView.baseTheme({
    ".cm-spell-error": { textDecoration: "underline wavy red" }
  });

  return [plugin, theme];
}
