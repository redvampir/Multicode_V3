# Layout export format

The visual editor can save and restore its state through JSON. Use the **Export** and **Import** buttons in the interface or call the methods directly on `VisualCanvas`.

## JSON structure

```json
{
  "blocks": [],
  "connections": [["a", "b"]],
  "offset": { "x": 0, "y": 0 },
  "scale": 1
}
```

- `blocks` — array of block metadata (`blocksData`).
- `connections` — list of edges as pairs of block `visual_id`s.
- `offset` — current pan offset of the canvas.
- `scale` — zoom level.

## Example

```js
// Export current layout
const json = JSON.stringify(vc.serialize(), null, 2);

// Restore layout
vc.load(JSON.parse(json));
```
