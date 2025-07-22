use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::path::Path;
use tracing::{Level, span, warn};

#[derive(Debug, Default)]
pub struct Section {
    entries: HashMap<Box<str>, Vec<Box<str>>>,
}

impl Section {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_all(&self, key: &str) -> Option<Vec<Box<str>>> {
        self.entries.get(key).cloned()
    }
    pub fn get_first(&self, key: &str) -> Option<Box<str>> {
        self.entries.get(key)?.first().cloned()
    }

    pub fn get_first_as_path(&self, key: &str) -> Option<Box<Path>> {
        self.get_first(key).as_deref().map(Path::new).map(Box::from)
    }

    pub fn get_first_as_boolean(&self, key: &str) -> Option<bool> {
        self.get_first(key).map(|s| &*s == "true")
    }
}

impl Section {
    pub fn insert_item(&mut self, mime: Box<str>, desktop_file: Box<str>) {
        self.entries.entry(mime).or_default().push(desktop_file);
    }
    pub fn insert_item_at_front(&mut self, mime: Box<str>, desktop_file: Box<str>) {
        self.entries
            .entry(mime)
            .or_default()
            .insert(0, desktop_file);
    }
    pub fn insert_items(&mut self, mime: Box<str>, mut desktop_files: Vec<Box<str>>) {
        self.entries
            .entry(mime)
            .or_default()
            .append(&mut desktop_files);
    }
}

#[derive(Debug, Default)]
pub struct IniFileOwned {
    sections: HashMap<Box<str>, Section>,
}

impl IniFileOwned {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(content: &str) -> IniFileOwned {
        let _span = span!(Level::TRACE, "from_str").entered();

        let mut sections = HashMap::new();
        let mut current_section = sections
            .entry(Box::from(""))
            .or_insert_with(Section::default);

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                let current_section_name = &line[1..line.len() - 1];
                current_section = sections
                    .entry(Box::from(current_section_name.trim()))
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
                let values = value
                    .split(';')
                    .filter(|s| !s.is_empty())
                    .map(|s| Box::from(s.trim()))
                    .collect::<Vec<_>>();
                current_section.insert_items(Box::from(key), values);
            } else {
                warn!("malformed line: {}", line);
            }
        }

        IniFileOwned { sections }
    }
}

impl IniFileOwned {
    pub fn get_section(&self, section_name: &str) -> Option<&Section> {
        self.sections.get(section_name)
    }

    pub fn sections(&self) -> &HashMap<Box<str>, Section> {
        &self.sections
    }

    pub fn format(&self) -> String {
        let mut str = String::with_capacity(self.into_iter().count() * 20); // 20 chars per line should be good
        let mut sections = self.sections().iter().collect::<Vec<_>>();
        sections.sort_by_key(|&(name, _)| name);
        for (name, section) in sections {
            if !name.is_empty() {
                if str.is_empty() {
                    str.push_str(&format!("[{name}]\n"));
                } else {
                    str.push_str(&format!("\n[{name}]\n"));
                }
            }
            let mut section = section.into_iter().collect::<Vec<_>>();
            section.sort_by_key(|(key, _)| *key);
            for (key, values) in section {
                str.push_str(&format!("{key}={}\n", values.join(";")));
            }
        }
        str
    }
}

impl IniFileOwned {
    pub fn get_section_mut(&mut self, section_name: &str) -> Option<&mut Section> {
        self.sections.get_mut(section_name)
    }

    pub fn section_entry(&mut self, section_name: Box<str>) -> Entry<'_, Box<str>, Section> {
        self.sections.entry(section_name)
    }
    pub fn insert_section(&mut self, name: Box<str>, section: Section) {
        self.sections.insert(name, section);
    }
}

impl<'a> IntoIterator for &'a IniFileOwned {
    type Item = (&'a Box<str>, &'a Box<str>, &'a Vec<Box<str>>);
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        let iter = self.sections.iter().flat_map(|(section_name, section)| {
            section
                .into_iter()
                .map(move |(key, values)| (section_name, key, values))
        });
        Box::new(iter)
    }
}

impl<'a> IntoIterator for &'a Section {
    type Item = (&'a Box<str>, &'a Vec<Box<str>>);
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        let iter = self.entries.iter();
        Box::new(iter)
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
key with spaces=value with spaces; and more values
"#;

        let ini = IniFileOwned::from_str(content);

        assert_eq!(
            ini.get_section("Section1").unwrap().get_first("key1"),
            Some("value1".into())
        );
        assert_eq!(
            ini.get_section("Section2").unwrap().get_first("foo"),
            Some("bar".into())
        );

        assert!(ini.get_section("Empty Section").is_some());
        assert_ne!(
            ini.get_section("Section With Spaces")
                .unwrap()
                .get_all("key with spaces"),
            Some(vec!["value with spaces".into()])
        );
        assert_ne!(
            ini.get_section("Section With Spaces")
                .unwrap()
                .get_all("key with spaces"),
            Some(vec!["value with spaces".into()])
        );
        assert_eq!(
            ini.get_section("Section With Spaces")
                .unwrap()
                .get_all("key with spaces"),
            Some(vec!["value with spaces".into(), "and more values".into()])
        );

        assert!(ini.get_section("NonExistent").is_none());
        assert_eq!(
            ini.get_section("Section1")
                .unwrap()
                .get_first("nonexistent"),
            None
        );
    }

    #[test]
    fn test_empty_ini() {
        let content = "";
        let ini = IniFileOwned::from_str(content);
        assert_eq!(ini.sections().len(), 1);
    }

    #[test]
    fn test_no_sections() {
        let content = "key=value";
        let ini = IniFileOwned::from_str(content);
        assert_eq!(
            ini.get_section("".into()).unwrap().get_first("key".into()),
            Some("value".into())
        );
    }

    #[test]
    fn test_values_iterator() {
        let content = r#"
    [Section1]
    key1=value1
    key2=value2;values3

    [Section2]
    foo=bar
    "#;
        let ini = IniFileOwned::from_str(content);
        let mut values: Vec<_> = ini.into_iter().collect();
        values.sort_by_key(|&(section, key, _)| (section, key));
        let mut iter = values.iter();
        assert_eq!(
            iter.next(),
            Some(&(&"Section1".into(), &"key1".into(), &vec!["value1".into()]))
        );
        assert_eq!(
            iter.next(),
            Some(&(
                &"Section1".into(),
                &"key2".into(),
                &vec!["value2".into(), "values3".into()]
            ))
        );
        assert_eq!(
            iter.next(),
            Some(&(&"Section2".into(), &"foo".into(), &vec!["bar".into()]))
        );
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
        let ini = IniFileOwned::from_str(content);
        let mut count = 0;
        for (section, name, value) in ini.into_iter() {
            assert!(!section.is_empty(), "Item should not be empty");
            assert!(!name.is_empty(), "Item should not be empty");
            assert!(!value.is_empty(), "Item should not be empty");
            count += 1;
        }
        assert_eq!(count, 3, "There should be 3 items in the iterator");
    }

    #[test]
    fn test_format_empty() {
        let content = "test=test";
        let ini = IniFileOwned::from_str(content);
        assert_eq!(ini.format(), "test=test\n");
    }

    #[test]
    fn test_format_multiple_sections() {
        let content = r#"[B]
key1=value1
key2=value2;value3

[A]
foo=bar
"#;
        let content2 = r#"[A]
foo=bar

[B]
key1=value1
key2=value2;value3
"#;
        let ini = IniFileOwned::from_str(content);
        assert_eq!(ini.format(), content2);
    }
}
