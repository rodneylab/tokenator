use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use inquire::Select;
use miette::{Context, IntoDiagnostic, bail};
use strsim::normalized_damerau_levenshtein;

#[derive(serde::Deserialize)]
struct Model {
    name: String,
    hf: String,
}

/// Loads the model name map from the JSON file (`data/models.json`) and returns it as a `HashMap`.
///
/// # Errors
/// Errors if:
/// - unable to read the file; or
/// - unable to parse the JSON content.
fn load_model_name_map<P: AsRef<Path>>(
    path: P,
) -> miette::Result<HashMap<String, String, ahash::RandomState>> {
    let data = fs::read_to_string(&path)
        .into_diagnostic()
        .wrap_err("Reading models JSON file")?;
    let models: Vec<Model> = serde_json::from_str(&data)
        .into_diagnostic()
        .wrap_err("Parsing models JSON file")?;
    if models.is_empty() {
        log::warn!("Models file `{}` is empty", path.as_ref().display());
    }

    let result = models.into_iter().map(|val| (val.name, val.hf)).collect();

    Ok(result)
}

/// Suggests a model name based on the input name.  Useful if the input name does not match any
/// available models.  Function logic is not optimised for large model name maps, and an
/// alternative data structure might be appropriate if the model set grows.
///
/// # Returns
/// An `Option` containing the suggested model name or `None` if no suggestion is found.
fn model_name_suggestion<'a>(
    model_name_map: &'a HashMap<String, String, ahash::RandomState>,
    input_name: &str,
) -> Option<&'a str> {
    model_name_map
        .iter()
        // returns an [`Option`] of the HashMap element with closest match (None if this fails)
        .max_by(|(key_a, _), (key_b, _)| {
            normalized_damerau_levenshtein(key_a, input_name)
                .partial_cmp(&normalized_damerau_levenshtein(key_b, input_name))
                .expect(
                    "distances should be in range [0,1] and so, partial_cmp should return `Some`",
                )
        })
        // maps option on closest HashMap element to `&str`
        .map(|(suggestion_key, _suggestion_hf)| suggestion_key.as_str())
}

/// Prompts the user to select a model name from a list.
///
/// # Returns
/// A `miette::Result` containing the user-selected model name.
fn get_model_name(
    model_name_map: &HashMap<String, String, ahash::RandomState>,
) -> miette::Result<String> {
    debug_assert!(!model_name_map.is_empty());
    let mut options: Vec<&String> = model_name_map.keys().collect();
    options.sort();

    let choice = Select::new("Which model are you using?", options)
        .prompt()
        .into_diagnostic()
        .wrap_err("Getting user model choice")?;

    Ok(choice.to_owned())
}

/// Retrieves the repository ID based on the model name.
///
/// # Returns
/// A `miette::Result` containing the repository ID.  Makes a suggestion if the given model name
/// does not exist.
///
/// # Errors
/// Errors if `model_name` does not match any existing models.
pub fn get_repo_id(
    model_name: Option<&String>,
    model_map_path: Option<PathBuf>,
) -> miette::Result<String> {
    let model_map_path = model_map_path
        .unwrap_or(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/models.json"));
    let model_name_map = load_model_name_map(&model_map_path)?;
    if model_name_map.is_empty() {
        bail!(
            "Error: no models in model file `{}`",
            model_map_path.display()
        );
    }
    let model_name = match model_name {
        Some(value) => value,
        None => &get_model_name(&model_name_map)?,
    };
    match model_name_map.get(model_name) {
        Some(value) => Ok(value.to_owned()),
        None => {
            if let Some(value) = model_name_suggestion(&model_name_map, model_name) {
                bail!("No model matching `{model_name}`, did you mean `{value}`?");
            } else {
                bail!("No model matching `{model_name}`.");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use assert_fs::{
        TempDir,
        prelude::{FileWriteStr, PathChild},
    };

    use crate::models::{get_repo_id, load_model_name_map, model_name_suggestion};

    #[test]
    fn load_model_name_map_generates_expected_output_from_valid_input() {
        // arrange
        let content = r#"[
  {
    "name": "example:latest",
    "hf": "example/Example-1-M-state-of-the-art"
  },
  {
    "name": "example:100b",
    "hf": "example/Example-100-B-most-capable"
  }
]
"#;
        let temp_dir = TempDir::new().unwrap();
        let _ = temp_dir.child("models.json").write_str(content);
        let temp_data_path = temp_dir.join("models.json");

        // act
        let outcome = load_model_name_map(temp_data_path).unwrap();

        // assert
        insta::assert_json_snapshot!(outcome, { "." => insta::sorted_redaction() });
    }

    #[test]
    fn load_model_name_map_returns_error_if_json_is_not_valid() {
        // arrange
        let content = r#"[
  {
    "name": "example:latest",
    "hf": "example/Example-1-M-state-of-the-art"
  },
  {
    "name": "example:100b",
    "hugging face": "example/Example-100-B-most-capable"
  }
]
"#;
        let temp_dir = TempDir::new().unwrap();
        let _ = temp_dir.child("models.json").write_str(content);
        let temp_data_path = temp_dir.join("models.json");

        // act
        let outcome = load_model_name_map(temp_data_path).unwrap_err();

        // assert
        let mut chain = outcome.chain();
        assert_eq!(
            chain.next().map(|val| format!("{val}")),
            Some("Parsing models JSON file".to_owned())
        );
        assert_eq!(
            chain.next().map(|val| format!("{val}")),
            Some("missing field `hf` at line 9 column 3".to_owned())
        );
        assert!(chain.next().is_none());

        // cleanup
        temp_dir.close().unwrap();
    }

    #[test]
    fn load_model_name_map_handles_empty_input() {
        // arrange
        let content = "[ ]";
        let temp_dir = TempDir::new().unwrap();
        let _ = temp_dir.child("models.json").write_str(content);
        let temp_data_path = temp_dir.join("models.json");

        // act
        let outcome = load_model_name_map(temp_data_path).unwrap();

        // assert
        assert!(outcome.is_empty());
    }

    #[test]
    fn model_name_suggestion_returns_closest_match() {
        // arrange
        let hasher = ahash::RandomState::new();
        let mut model_name_map: HashMap<String, String, ahash::RandomState> =
            HashMap::with_hasher(hasher);
        model_name_map.insert(
            "example-model".to_owned(),
            "example/Example-Model".to_owned(),
        );
        model_name_map.insert(
            "nothing-to-do-with-the-other-one".to_owned(),
            "example/TheOtherExample".to_owned(),
        );
        model_name_map.insert(
            "example-model:7b".to_owned(),
            "example/Example-7-B".to_owned(),
        );
        let input_name = "example_model";

        // act
        let outcome = model_name_suggestion(&model_name_map, input_name).unwrap();

        // assert
        assert_eq!(outcome, "example-model");
    }

    #[test]
    fn get_repo_id_generates_expected_result_with_valid_input() {
        // arrange
        let content = r#"[
  {
    "name": "example-model",
    "hf": "example/Example-Model"
  },
  {
    "name": "nothing-to-do-with-the-other-one",
    "hf": "example/TheOtherExample"
  }
]
"#;
        let temp_dir = TempDir::new().unwrap();
        let _ = temp_dir.child("models.json").write_str(content);
        let temp_data_path = temp_dir.join("models.json");

        // act
        let outcome =
            get_repo_id(Some(&String::from("example-model")), Some(temp_data_path)).unwrap();

        // assert
        assert_eq!(outcome, "example/Example-Model");
    }

    #[test]
    fn get_repo_id_generates_expected_error_with_invalid_input() {
        // arrange
        let content = r#"[
  {
    "name": "example-model",
    "hf": "example/Example-Model"
  },
  {
    "name": "nothing-to-do-with-the-other-one",
    "hf": "example/TheOtherExample"
  }
]
"#;
        let temp_dir = TempDir::new().unwrap();
        let _ = temp_dir.child("models.json").write_str(content);
        let temp_data_path = temp_dir.join("models.json");

        // act
        let outcome =
            get_repo_id(Some(&String::from("example-modal")), Some(temp_data_path)).unwrap_err();

        // assert
        let mut chain = outcome.chain();
        assert_eq!(
            chain.next().map(|val| format!("{val}")),
            Some("No model matching `example-modal`, did you mean `example-model`?".to_owned())
        );
        assert!(chain.next().is_none());

        // cleanup
        temp_dir.close().unwrap();
    }

    #[test]
    fn get_repo_id_generates_expected_error_with_empty_model_name_map() {
        // arrange
        let content = "[ ]";
        let temp_dir = TempDir::new().unwrap();
        let _ = temp_dir.child("models.json").write_str(content);
        let temp_data_path = temp_dir.join("models.json");

        // act
        let outcome = get_repo_id(
            Some(&String::from("example-modal")),
            Some(temp_data_path.clone()),
        )
        .unwrap_err();

        // assert
        let mut chain = outcome.chain();
        assert_eq!(
            chain.next().map(|val| format!("{val}")),
            Some(format!(
                "Error: no models in model file `{}`",
                temp_data_path.display()
            ))
        );
        assert!(chain.next().is_none());

        // cleanup
        temp_dir.close().unwrap();
    }
}
