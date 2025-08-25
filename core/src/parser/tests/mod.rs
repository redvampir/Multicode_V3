use super::*;
use std::collections::HashSet;

#[test]
fn lang_display_and_from_str() {
    let cases = [
        (Lang::Rust, "rust"),
        (Lang::Python, "python"),
        (Lang::JavaScript, "javascript"),
        (Lang::Css, "css"),
        (Lang::Html, "html"),
        (Lang::Go, "go"),
        (Lang::TypeScript, "typescript"),
        (Lang::C, "c"),
        (Lang::Cpp, "cpp"),
        (Lang::Java, "java"),
        (Lang::CSharp, "csharp"),
    ];
    for (lang, name) in cases {
        assert_eq!(lang.to_string(), name);
        assert_eq!(name.parse::<Lang>().ok(), Some(lang));
    }
}

#[test]
fn parse_sources_into_blocks() {
    let cases = [
        (Lang::Rust, "fn main() { println!(\"hi\"); }"),
        (Lang::Python, "def main():\n    print('hi')"),
        (Lang::JavaScript, "function main() { console.log('hi'); }"),
        (Lang::Css, "body { color: red; }"),
        (Lang::Html, "<html></html>"),
        (Lang::Go, "package main\nfunc main() { println(\"hi\") }"),
        (
            Lang::TypeScript,
            "function main(): void { console.log('hi'); }",
        ),
        (Lang::C, "int main() { return 0; }"),
        (Lang::Cpp, "int main() { return 0; }"),
        (
            Lang::Java,
            "class Main { public static void main(String[] args) { } }",
        ),
        (
            Lang::CSharp,
            "class Program { static void Main(string[] args) { } }",
        ),
    ];

    for (lang, source) in cases {
        let tree = parse(source, lang, None).expect("не удалось разобрать");
        let blocks = parse_to_blocks(&tree, None);
        assert!(!blocks.is_empty());
        let mut unique = HashSet::new();
        for block in &blocks {
            assert!(unique.insert(block.node_id));
            assert!(!block.visual_id.is_empty());
        }
    }
}

#[test]
fn parse_expression_into_ops_and_variables() {
    let src = "a + b * c";
    let tree = parse(src, Lang::Python, None).expect("не удалось разобрать");
    let blocks = parse_to_blocks(&tree, None);
    let mut found = 0;
    for b in &blocks {
        match b.kind.as_str() {
            "Op/+" => {
                assert_eq!(b.anchors, vec![(2, 3)]);
                found += 1;
            }
            "Op/*" => {
                assert_eq!(b.anchors, vec![(6, 7)]);
                found += 1;
            }
            "Variable/Get" => {
                if b.anchors == vec![(0, 1)]
                    || b.anchors == vec![(4, 5)]
                    || b.anchors == vec![(8, 9)]
                {
                    found += 1;
                }
            }
            _ => {}
        }
    }
    assert_eq!(found, 5);
}

#[test]
fn parse_ternary_expression_into_op() {
    let src = "a ? b : c";
    let tree = parse(src, Lang::JavaScript, None).expect("не удалось разобрать");
    let blocks = parse_to_blocks(&tree, None);
    assert!(blocks.iter().any(|b| b.kind == "Op/Ternary"));
}

fn assert_example(lang: Lang, source: &str) {
    let tree = parse(source, lang, None).expect("не удалось разобрать");
    assert!(tree.root_node().child_count() > 0);
    let blocks = parse_to_blocks(&tree, None);
    assert!(!blocks.is_empty());
    assert!(blocks.iter().all(|b| !b.visual_id.is_empty()));
}

#[test]
fn c_example_has_visual_ids() {
    assert_example(Lang::C, "int main() { return 0; }");
}

#[test]
fn cpp_example_has_visual_ids() {
    assert_example(Lang::Cpp, "int main() { return 0; }");
}

#[test]
fn java_example_has_visual_ids() {
    assert_example(
        Lang::Java,
        "class Main { public static void main(String[] args) {} }",
    );
}

#[test]
fn csharp_example_has_visual_ids() {
    assert_example(
        Lang::CSharp,
        "class Program { static void Main(string[] args) { } }",
    );
}
