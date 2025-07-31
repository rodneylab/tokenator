use rexpect::spawn;

#[test]
fn get_model_name_displays_model_list() {
    let mut p = spawn("./target/debug/tokenator", Some(1_000)).unwrap();
    p.exp_regex("Which model are you using?").unwrap();
}
