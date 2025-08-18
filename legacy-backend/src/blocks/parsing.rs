use tree_sitter::{InputEdit, Point};

use crate::{get_document_tree, update_document_tree};
use crate::parser::{parse as ts_parse, parse_to_blocks, Block, Lang};

/// Parse source `content` of language `lang` into syntax `Block`s.
///
/// Reuses the previously stored parse tree for incremental parsing when
/// available, updating the cached tree after parsing.
pub fn parse(content: &str, lang: Lang) -> Option<Vec<Block>> {
    let old = get_document_tree("current");
    let tree = if let Some(mut old_tree) = old {
        let old_root = old_tree.root_node();
        let old_end_byte = old_root.end_byte();
        let old_end_position = old_root.end_position();
        let new_end_byte = content.as_bytes().len();
        let mut row = 0;
        let mut column = 0;
        for b in content.bytes() {
            if b == b'\n' {
                row += 1;
                column = 0;
            } else {
                column += 1;
            }
        }
        let new_end_position = Point { row, column };
        let edit = InputEdit {
            start_byte: 0,
            old_end_byte,
            new_end_byte,
            start_position: Point { row: 0, column: 0 },
            old_end_position,
            new_end_position,
        };
        old_tree.edit(&edit);
        ts_parse(content, lang, Some(&old_tree))?
    } else {
        ts_parse(content, lang, None)?
    };
    update_document_tree("current".to_string(), tree.clone());
    Some(parse_to_blocks(&tree))
}
