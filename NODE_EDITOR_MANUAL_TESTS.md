# Node Editor Manual Test Scenarios

These scenarios help to manually verify core functionality of the node editor.

## 1. Block Parsing and Display
1. Open the application with a sample source file.
2. Ensure blocks appear for major constructs (functions, variables, conditions, loops).
3. Change locale to Russian and Spanish; block labels should update accordingly.

## 2. Synchronization of Positions
1. Drag a block to a new position.
2. Save and reload the project.
3. Verify the block stays at the moved position.
4. Use undo/redo buttons to ensure movement history is tracked.

## 3. Metadata Editing
1. Select a block and edit its translations.
2. Confirm changes appear on the canvas after closing the editor.
3. Add an AI note and hover the block to view the tooltip.

## 4. Connections
1. Create a connection between two blocks.
2. Move one block and ensure the connection updates.
3. Delete a block and check that associated connections are removed.

## 5. Export
1. Use the export command to save cleaned source code.
2. Open the exported file and verify no `@VISUAL_META` comments remain.

## 6. Import/Parsing
1. Load a file containing `@VISUAL_META` comments.
2. Ensure blocks appear at the correct positions with the stored translations.

## 7. WebSocket Synchronization
1. Open the editor in two windows.
2. Move a block in the first window and verify the change appears in the second.

These steps provide a baseline for manual validation of the editor's parsing, synchronization, and export features.
