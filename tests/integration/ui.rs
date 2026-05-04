use rexpect::spawn;

#[test]
fn displays_expected_prompt_token_count_reading_code_from_stdin() {
    let mut p = spawn(
        r#"./target/debug/tokenator 'println!("Made it here!");'"#,
        Some(5_000),
    )
    .unwrap();
    p.exp_regex("Which model are you using?").unwrap();
    p.send_line("qwen2.5-coder:7b").unwrap();
    p.exp_regex("Prompt token count: 6").unwrap();
}

#[test]
fn prompts_with_an_expected_model_name() {
    let mut p = spawn(
        r#"./target/debug/tokenator 'console.log("Made it here!");'"#,
        Some(5_000),
    )
    .unwrap();
    p.exp_regex("glm-4.7-flash").unwrap();
    p.send_line("glm-4.7-flash").unwrap();
    p.exp_regex("Prompt token count: 7").unwrap();
}
