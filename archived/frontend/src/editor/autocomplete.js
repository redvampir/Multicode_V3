// Store current block identifiers for suggestions
let blockIds = [];

// Simple list of keywords offered in completion
const keywords = [
  "function",
  "return",
  "const",
  "let",
  "if",
  "else",
  "for",
  "while"
];

export function setBlockIds(ids) {
  blockIds = Array.isArray(ids) ? ids : [];
}

// Custom completion source combining block ids and keywords
export const customSource = context => {
  const word = context.matchBefore(/\w*/);
  if (!word || (word.from === word.to && !context.explicit)) return null;
  const options = [
    ...blockIds.map(id => ({ label: id, type: "variable" })),
    ...keywords.map(kw => ({ label: kw, type: "keyword" }))
  ];
  return {
    from: word.from,
    options,
    validFor: /^\w*$/
  };
};
