use rexpect::spawn;

#[test]
fn get_model_name_displays_model_list() {
    let mut p = spawn(
        r#"./target/debug/tokenator 'println!("Made it here!");'"#,
        Some(5_000),
    )
    .unwrap();
    p.exp_regex("Which model are you using?").unwrap();
    p.send_line("qwen2.5-coder:7b").unwrap();
    p.exp_regex("Prompt token count: 6").unwrap();
}
