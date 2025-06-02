use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Default)]
pub struct IniFile<'a> {
    sections: HashMap<&'a str, Section<'a>>,
}

#[derive(Debug, Default)]
pub struct Section<'a> {
    entries: HashMap<&'a str, &'a str>,
}

impl<'a> Section<'a> {
    pub fn insert(&mut self, key: &'a str, value: &'a str) {
        self.entries.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&'a str> {
        self.entries.get(key).copied()
    }

    pub fn get_boxed(&self, key: &str) -> Option<Box<str>> {
        self.get(key).map(Box::from)
    }

    pub fn get_path_boxed(&self, key: &str) -> Option<Box<Path>> {
        self.get(key).map(Path::new).map(Box::from)
    }

    pub fn get_boolean(&self, key: &str) -> Option<bool> {
        self.get(key).map(|s| s == "true")
    }

    pub fn values(&'a self) -> impl Iterator<Item = &'a str> {
        self.entries.values().copied()
    }
}

impl<'a> IniFile<'a> {
    pub fn parse(content: &'a str) -> Self {
        let mut sections = HashMap::new();
        let mut current_section = sections.entry("").or_insert_with(Section::default);

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                let current_section_name = &line[1..line.len() - 1];
                current_section = sections
                    .entry(current_section_name.trim())
                    .or_insert_with(Section::default);
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                // Skip localized entries (containing [...])
                if key.contains('[') {
                    continue;
                }
                current_section.insert(key, value);
            }
        }

        Self { sections }
    }

    pub fn get_section(&self, section: &str) -> Option<&Section> {
        self.sections.get(section)
    }

    pub fn get_value(&self, section: &str, key: &str) -> Option<&'a str> {
        self.sections.get(section).and_then(|s| s.get(key))
    }

    pub fn sections(&self) -> &HashMap<&'a str, Section> {
        &self.sections
    }

    pub fn values(&'a self) -> impl Iterator<Item = &'a str> + 'a {
        self.sections.values().flat_map(|section| section.values())
    }
}

impl<'a> IntoIterator for &'a IniFile<'a> {
    type Item = &'a str;
    type IntoIter = Box<dyn Iterator<Item = &'a str> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.values())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ini() {
        let content = r#"[Section1]
key1=value1
key2=value2

[Section2]
foo=bar
baz=qux

; Comment
# Another comment
[Empty Section]

[Section With Spaces]
key with spaces=value with spaces
"#;

        let ini = IniFile::parse(content);

        // Test section content
        assert_eq!(ini.get_value("Section1", "key1"), Some("value1"));
        assert_eq!(ini.get_value("Section2", "foo"), Some("bar"));

        // Test section existence
        assert!(ini.get_section("Empty Section").is_some());

        // Test spaces in keys and values
        assert_eq!(
            ini.get_value("Section With Spaces", "key with spaces"),
            Some("value with spaces")
        );

        // Test non-existent sections and keys
        assert_eq!(ini.get_value("NonExistent", "key"), None);
        assert_eq!(ini.get_value("Section1", "nonexistent"), None);
    }

    #[test]
    fn test_empty_ini() {
        let content = "";
        let ini = IniFile::parse(content);
        assert_eq!(ini.sections().len(), 1);
    }

    #[test]
    fn test_no_sections() {
        let content = "key=value";
        let ini = IniFile::parse(content);
        assert_eq!(ini.get_value("", "key"), Some("value"));
    }

    #[test]
    fn test_values_iterator() {
        let content = r#"
    [Section1]
    key1=value1
    key2=value2

    [Section2]
    foo=bar
    "#;
        let ini = IniFile::parse(content);
        let values: Vec<_> = ini.values().collect();
        assert!(values.contains(&"value1"));
        assert!(values.contains(&"value2"));
        assert!(values.contains(&"bar"));
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_values_iterator_2() {
        let content = r#"
    [Section1]
    key1=value1
    key2=value2

    [Section2]
    foo=bar
    "#;
        let ini = IniFile::parse(content);
        let mut count = 0;
        for item in &ini {
            assert!(!item.is_empty(), "Item should not be empty");
            count += 1;
        }
        assert_eq!(count, 3, "There should be 3 items in the iterator");
    }
}
