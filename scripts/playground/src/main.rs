
fn main() {
    // 1) rfd before SoLoud init
    let path_before = rfd::FileDialog::new()
        .set_title("Before SoLoud")
        .pick_file();
    eprintln!("Dialog BEFORE SoLoud: {path_before:?}");

    // 2) Initialize SoLoud
    let _sl = soloud::Soloud::default().expect("Failed to create SoLoud");

    // 3) rfd after SoLoud init
    let path_after = rfd::FileDialog::new()
        .set_title("After SoLoud")
        .pick_file();
    eprintln!("Dialog AFTER SoLoud: {path_after:?}");
}
