use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt::Write;
use std::path::Path;
use tracing::{debug_span, warn};

#[derive(Debug, Default)]
pub struct Section<'a> {
    entries: HashMap<&'a str, Vec<&'a str>>,
}

impl<'a> Section<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_all(&self, key: &str) -> Option<&Vec<&'a str>> {
        self.entries.get(key)
    }
    pub fn get_first(&self, key: &str) -> Option<&'a str> {
        self.entries.get(key)?.first().copied()
    }

    pub fn get_all_as_boxed(&self, key: &str) -> Option<Vec<Box<str>>> {
        self.get_all(key)
            .map(|vec| vec.iter().copied().map(Box::from).collect::<Vec<_>>())
    }

    pub fn get_first_as_boxed(&self, key: &str) -> Option<Box<str>> {
        self.get_first(key).map(Box::from)
    }

    pub fn get_first_as_path_boxed(&self, key: &str) -> Option<Box<Path>> {
        self.get_first(key).map(Path::new).map(Box::from)
    }

    pub fn get_first_as_boolean(&self, key: &str) -> Option<bool> {
        self.get_first(key).map(|s| s == "true")
    }
}

impl<'a> Section<'a> {
    pub fn insert_item(&mut self, key: &'a str, desktop_file: &'a str) {
        self.entries.entry(key).or_default().push(desktop_file);
    }
    pub fn insert_item_at_front(&mut self, key: &'a str, desktop_file: &'a str) {
        self.entries.entry(key).or_default().insert(0, desktop_file);
    }
    pub fn insert_items(&mut self, key: &'a str, mut desktop_files: Vec<&'a str>) {
        self.entries
            .entry(key)
            .or_default()
            .append(&mut desktop_files);
    }
    pub fn set_items(&mut self, key: &'a str, mut desktop_files: Vec<&'a str>) {
        self.entries
            .entry(key)
            .and_modify(|e| {
                e.clear();
                e.append(&mut desktop_files);
            })
            .or_insert_with(|| desktop_files);
    }
}

#[derive(Debug, Default)]
pub struct IniFile<'a> {
    sections: HashMap<&'a str, Section<'a>>,
}

impl IniFile<'_> {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(content: &str) -> IniFile {
        let _span = debug_span!("from_str").entered();

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
                let values = value
                    .split(';')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>();
                current_section.insert_items(key, values);
            } else {
                warn!("malformed line: {}", line);
            }
        }

        IniFile { sections }
    }
}

impl<'a> IniFile<'a> {
    pub fn get_section(&'a self, section_name: &str) -> Option<&'a Section<'a>> {
        self.sections.get(section_name)
    }

    pub const fn sections(&self) -> &HashMap<&'a str, Section<'a>> {
        &self.sections
    }

    pub fn format(&self) -> String {
        let mut str = String::with_capacity(self.into_iter().count() * 20); // 20 chars per line should be good
        let mut sections = self.sections().iter().collect::<Vec<_>>();
        sections.sort_by_key(|&(name, _)| name);
        for (name, section) in sections {
            if !name.is_empty() {
                if str.is_empty() {
                    let _ = str.write_str(&format!("[{name}]\n"));
                } else {
                    let _ = str.write_str(&format!("\n[{name}]\n"));
                }
            }
            let mut section = section.into_iter().collect::<Vec<_>>();
            section.sort_by_key(|(key, _)| *key);
            for (key, values) in section {
                let _ = str.write_str(&format!("{key}={}\n", values.join(";")));
            }
        }
        str
    }
}

impl<'a> IniFile<'a> {
    pub fn get_section_mut<'b>(&'b mut self, section_name: &str) -> Option<&'b mut Section<'a>>
    where
        'a: 'b,
    {
        self.sections.get_mut(section_name)
    }

    pub fn section_entry<'b>(&'b mut self, section_name: &'a str) -> Entry<'b, &'a str, Section<'a>>
    where
        'a: 'b,
    {
        self.sections.entry(section_name)
    }
    pub fn insert_section(&mut self, name: &'a str, section: Section<'a>) {
        self.sections.insert(name, section);
    }
}

impl<'a> IniFile<'a> {
    #[allow(dead_code)]
    fn iter(&'a self) -> Box<dyn Iterator<Item = <&'a Self as IntoIterator>::Item> + 'a> {
        <&Self as IntoIterator>::into_iter(self)
    }
}

impl<'a> IntoIterator for &'a IniFile<'a> {
    type Item = (&'a str, &'a str, &'a Vec<&'a str>);
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        let iter = self.sections.iter().flat_map(|(section_name, section)| {
            section
                .into_iter()
                .map(move |(key, values)| (*section_name, key, values))
        });
        Box::new(iter)
    }
}

impl<'a> Section<'a> {
    #[allow(dead_code)]
    fn iter(&'a self) -> Box<dyn Iterator<Item = <&'a Self as IntoIterator>::Item> + 'a> {
        <&Self as IntoIterator>::into_iter(self)
    }
}

impl<'a> IntoIterator for &'a Section<'a> {
    type Item = (&'a str, &'a Vec<&'a str>);
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        let iter = self.entries.iter().map(|(key, value)| (*key, value));
        Box::new(iter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ini() {
        let content = r"[Section1]
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
";

        let ini = IniFile::from_str(content);

        assert_eq!(
            ini.get_section("Section1").unwrap().get_first("key1"),
            Some("value1")
        );
        assert_eq!(
            ini.get_section("Section2").unwrap().get_first("foo"),
            Some("bar")
        );

        assert!(ini.get_section("Empty Section").is_some());
        assert_ne!(
            ini.get_section("Section With Spaces")
                .unwrap()
                .get_all("key with spaces"),
            Some(&vec!["value with spaces"])
        );
        assert_ne!(
            ini.get_section("Section With Spaces")
                .unwrap()
                .get_all("key with spaces"),
            Some(&vec!["value with spaces"])
        );
        assert_eq!(
            ini.get_section("Section With Spaces")
                .unwrap()
                .get_all("key with spaces"),
            Some(&vec!["value with spaces", "and more values"])
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
        let ini = IniFile::from_str(content);
        assert_eq!(ini.sections().len(), 1);
    }

    #[test]
    fn test_no_sections() {
        let content = "key=value";
        let ini = IniFile::from_str(content);
        assert_eq!(ini.get_section("").unwrap().get_first("key"), Some("value"));
    }

    #[test]
    fn test_values_iterator() {
        let content = r"
    [Section1]
    key1=value1
    key2=value2;values3

    [Section2]
    foo=bar
    ";
        let ini = IniFile::from_str(content);
        let mut values: Vec<_> = ini.into_iter().collect();
        values.sort_by_key(|&(section, key, _)| (section, key));
        let mut iter = values.iter();
        assert_eq!(iter.next(), Some(&("Section1", "key1", &vec!["value1"])));
        assert_eq!(
            iter.next(),
            Some(&("Section1", "key2", &vec!["value2", "values3"]))
        );
        assert_eq!(iter.next(), Some(&("Section2", "foo", &vec!["bar"])));
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_values_iterator_2() {
        let content = r"
    [Section1]
    key1=value1
    key2=value2

    [Section2]
    foo=bar
    ";
        let ini = IniFile::from_str(content);
        let mut count = 0;
        for (section, name, value) in &ini {
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
        let ini = IniFile::from_str(content);
        assert_eq!(ini.format(), "test=test\n");
    }

    #[test]
    fn test_format_multiple_sections() {
        let content = r"[B]
key1=value1
key2=value2;value3

[A]
foo=bar
";
        let content2 = r"[A]
foo=bar

[B]
key1=value1
key2=value2;value3
";
        let ini = IniFile::from_str(content);
        assert_eq!(ini.format(), content2);
    }
}
