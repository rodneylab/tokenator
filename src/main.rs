#![warn(clippy::all, clippy::pedantic)]

mod cli;
mod models;
mod prompt;
mod token;
mod utility;

use clap::Parser;
use num_format::Locale;

use crate::{
    cli::Cli,
    models::get_repo_id,
    prompt::get_prompt,
    token::{count_tokens, create_tokeniser},
};

fn format_number(number: usize) -> String {
    let mut buf = num_format::Buffer::default();
    buf.write_formatted(&number, &Locale::en);

    buf.as_str().to_owned()
}

/// Main function to run the token counting tool.
fn main() -> miette::Result<()> {
    let cli = &Cli::parse();
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();
    let Cli {
        model,
        file,
        prompt,
        ..
    } = cli;

    let repo_id = get_repo_id(model.as_ref(), None)?;
    let tokeniser = create_tokeniser(&repo_id)?;
    let prompt = get_prompt(file.clone(), prompt.as_deref())?;
    let tokens = count_tokens(&tokeniser, &prompt)?;

    println!("Prompt token count: {}", format_number(tokens));

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::format_number;

    #[test]
    fn format_number_generates_expected_output_for_valid_input() {
        // arrange
        let number = 42;

        // act
        let outcome = format_number(number);

        // assert
        assert_eq!(&outcome, "42");

        // arrange
        let number = 10_000_000_000;

        // act
        let outcome = format_number(number);

        // assert
        assert_eq!(&outcome, "10,000,000,000");

        // arrange
        let number = 1_000_000;

        // act
        let outcome = format_number(number);

        // assert
        assert_eq!(&outcome, "1,000,000");

        // arrange
        let number = 0;

        // act
        let outcome = format_number(number);

        // assert
        assert_eq!(&outcome, "0");
    }
}
