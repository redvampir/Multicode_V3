use desktop::ui::MainUI;

#[test]
fn main_ui_smoke_test() {
    let ui = MainUI::default();
    ui.view();
}
