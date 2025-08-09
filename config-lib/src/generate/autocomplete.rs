use inquire::{Autocomplete, CustomUserError};

#[derive(Debug, Clone, Default)]
pub struct StringAutoCompleter {
    strings: Box<[Box<str>]>,
}

impl From<Vec<&str>> for StringAutoCompleter {
    fn from(strings: Vec<&str>) -> Self {
        Self {
            strings: strings
                .into_iter()
                .map(|s| s.to_string().into_boxed_str())
                .collect(),
        }
    }
}

impl From<Box<[&str]>> for StringAutoCompleter {
    fn from(value: Box<[&str]>) -> Self {
        Self {
            strings: value
                .iter()
                .map(|s| (*s).to_string().into_boxed_str())
                .collect(),
        }
    }
}

impl Autocomplete for StringAutoCompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        Ok(self
            .strings
            .iter()
            .filter(|terminal| terminal.starts_with(input))
            .map(ToString::to_string)
            .collect())
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Option<String>, CustomUserError> {
        Ok(if let Some(suggestion) = highlighted_suggestion {
            Some(suggestion)
        } else {
            self.strings
                .iter()
                .find(|terminal| terminal.starts_with(input))
                .map(ToString::to_string)
        })
    }
}
