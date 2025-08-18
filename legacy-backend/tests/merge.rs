use backend::blocks::parse_blocks;

#[test]
fn stable_ids_preserved_when_appending() {
    let src1 = "fn a() {}\nfn b() {}\n".to_string();
    let blocks1 = parse_blocks(src1.clone(), "rust".into()).expect("first parse");
    let fns1: Vec<_> = blocks1
        .iter()
        .filter(|b| b.kind == "Function/Define")
        .collect();
    assert!(fns1.len() >= 2);

    let src2 = format!("{}fn c() {{}}\n", src1);
    let blocks2 = parse_blocks(src2, "rust".into()).expect("second parse");
    let fns2: Vec<_> = blocks2
        .iter()
        .filter(|b| b.kind == "Function/Define")
        .collect();
    assert!(fns2.len() >= 3);

    assert_eq!(fns1[0].visual_id, fns2[0].visual_id);
    assert_eq!(fns1[1].visual_id, fns2[1].visual_id);
}
