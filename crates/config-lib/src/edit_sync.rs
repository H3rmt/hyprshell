use anyhow::{Context, bail};
use toml_edit::{DocumentMut, Item, Table, TableLike, Value};

/// Syncronize src into destination
pub fn sync_document(dst: &mut DocumentMut, src: &DocumentMut) {
    sync(dst.as_table_mut(), src.as_table());
}

/// Must be called seperately on all arrays in a struct
pub fn sync_array<F, T>(dst: &mut Item, src: &Item, identity: F) -> anyhow::Result<()>
where
    F: Fn(&dyn TableLike) -> T,
    T: Eq + Clone + std::hash::Hash,
{
    #[cfg(debug_assertions)]
    tracing::trace!("Syncing array of tables: dst = {:?}, src = {:?}", dst, src);
    let mut exist = std::collections::HashSet::new();
    for src_el in src.iter_tables().context("invalid source")? {
        let id = identity(src_el);
        exist.insert(id.clone());
        if let Some(dst_el) = dst
            .iter_tables_mut()
            .context("invalid destination")?
            .find(|entry| identity(*entry) == id)
        {
            sync(dst_el, src_el);
        } else {
            dst.push_table(src_el).context("invalid destination")?;
        }
    }
    dst.retain(|el| exist.contains(&identity(el)))
        .context("invalid destination")?;

    Ok(())
}

fn sync(dst: &mut dyn TableLike, src: &dyn TableLike) {
    for (key, src_item) in src.iter() {
        let dst_item = dst.entry(key).or_insert(Item::None);
        sync_item(dst_item, src_item);
    }
}

fn sync_item(dst: &mut Item, src: &Item) {
    match (dst, src) {
        (Item::Table(dst), Item::Table(src)) => {
            #[cfg(debug_assertions)]
            tracing::trace!("Syncing table: dst = {:?}, src = {:?}", dst, src);
            sync(dst, src);
        }
        // happens when parsing struct into a DocumentMut, which uses inline tables for structs
        (Item::Table(dst), Item::Value(Value::InlineTable(src))) => {
            #[cfg(debug_assertions)]
            tracing::trace!(
                "Syncing table (from inline table): dst = {:?}, src = {:?}",
                dst,
                src
            );
            sync(dst, src);
        }
        // For anything else, fall back to replacement.
        (dst, src) if let Item::None = dst => {
            #[cfg(debug_assertions)]
            tracing::trace!("Replacing item: dst = None, src = {:?}", src);
            *dst = src.clone();
        }
        // For scalar values, just replace the value in place.
        (Item::Value(dst), Item::Value(src)) => {
            #[cfg(debug_assertions)]
            tracing::trace!("Syncing value: dst = {:?}, src = {:?}", dst, src);
            *dst = src.clone();
        }
        // (a, b) if a.into_array_of_tables().is_ok() || b.into_array_of_tables().is_ok() => {
        //     // skipped, must be manually done using sync_array
        // }
        (dst, src) => {
            #[cfg(debug_assertions)]
            tracing::trace!("Replacing item: dst = {:?}, src = {:?}", dst, src);
            panic!("items dont match");
        }
    }
}

trait ArrayOfTablesLike {
    fn iter_tables<'a>(
        &'a self,
    ) -> anyhow::Result<Box<dyn Iterator<Item = &'a dyn TableLike> + 'a>>;

    fn iter_tables_mut<'a>(
        &'a mut self,
    ) -> anyhow::Result<Box<dyn Iterator<Item = &'a mut dyn TableLike> + 'a>>;

    fn push_table(&mut self, table: &dyn TableLike) -> anyhow::Result<()>;

    fn retain<F: Fn(&dyn TableLike) -> bool>(&mut self, keep: F) -> anyhow::Result<()>;
}

impl ArrayOfTablesLike for Item {
    fn iter_tables<'a>(
        &'a self,
    ) -> anyhow::Result<Box<dyn Iterator<Item = &'a dyn TableLike> + 'a>> {
        match self {
            Self::None => bail!("not array of tables"),
            Self::Table(_) => bail!("not array of tables"),
            Self::Value(Value::Array(array)) => {
                Ok(Box::new(array.iter().filter_map(|el| {
                    el.as_inline_table().map(|t| t as &dyn TableLike)
                })))
            }
            Self::Value(_) => bail!("not array of tables"),
            Self::ArrayOfTables(array_of_tables) => Ok(Box::new(
                array_of_tables.iter().map(|t| t as &dyn TableLike),
            )),
        }
    }
    fn iter_tables_mut<'a>(
        &'a mut self,
    ) -> anyhow::Result<Box<dyn Iterator<Item = &'a mut dyn TableLike> + 'a>> {
        match self {
            Self::None => bail!("not array of tables"),
            Self::Table(_) => bail!("not array of tables"),
            Self::Value(Value::Array(array)) => {
                Ok(Box::new(array.iter_mut().filter_map(|el| {
                    el.as_inline_table_mut().map(|t| t as &mut dyn TableLike)
                })))
            }
            Self::Value(_) => bail!("not array of tables"),
            Self::ArrayOfTables(array_of_tables) => Ok(Box::new(
                array_of_tables.iter_mut().map(|t| t as &mut dyn TableLike),
            )),
        }
    }

    fn push_table(&mut self, table: &dyn TableLike) -> anyhow::Result<()> {
        match self {
            Self::None => bail!("not array of tables"),
            Self::Table(_) => bail!("not array of tables"),
            Self::Value(Value::Array(array)) => {
                array.push((Table::from_iter(table.iter())).into_inline_table())
            }
            Self::Value(_) => bail!("not array of tables"),
            Self::ArrayOfTables(array_of_tables) => {
                array_of_tables.push(Table::from_iter(table.iter()));
            }
        }
        Ok(())
    }

    fn retain<F: Fn(&dyn TableLike) -> bool>(&mut self, keep: F) -> anyhow::Result<()> {
        match self {
            Self::None => bail!("not array of tables"),
            Self::Table(_) => bail!("not array of tables"),
            Self::Value(Value::Array(array)) => {
                array.retain(|e| match e {
                    Value::InlineTable(table) => keep(table),
                    _ => false,
                });
            }
            Self::Value(_) => bail!("not array of tables"),
            Self::ArrayOfTables(array_of_tables) => {
                array_of_tables.retain(|e| keep(e));
            }
        }
        Ok(())
    }
}

#[allow(clippy::needless_raw_string_hashes)]
#[cfg(test)]
mod tests {
    use super::*;
    use toml_edit::DocumentMut;

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_sync_document_0() {
        let mut dst: DocumentMut = "# test comment\n[section]\nkey1 = \"value1\"\n"
            .parse()
            .unwrap();
        let src = "[section]\nkey1 = \"new_value\"\nkey2 = \"value2\"\n"
            .parse::<DocumentMut>()
            .unwrap();
        sync_document(&mut dst, &src);
        assert_eq!(
            dst.to_string(),
            "# test comment\n[section]\nkey1 = \"new_value\"\nkey2 = \"value2\"\n"
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_sync_document_1() {
        let mut dst: DocumentMut =
            "# test comment\n[section]\nkey1 = \"value1\"\nkey2 = \"value2\"\n"
                .parse()
                .unwrap();
        let src = "[section]\nkey1 = \"new_value\"\nkey2 = \"value3\"\n"
            .parse::<DocumentMut>()
            .unwrap();
        sync_document(&mut dst, &src);
        assert_eq!(
            dst.to_string(),
            "# test comment\n[section]\nkey1 = \"new_value\"\nkey2 = \"value3\"\n"
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_sync_document_2() {
        let mut dst: DocumentMut =
            "# test comment\n\nkey1 = \"value1\"\nkey2 = \"value2\"\n#test comment"
                .parse()
                .unwrap();
        let src = "\nkey1 = \"new_value\"\nkey2 = \"value3\"\n[section]\nkey = \"test\"\n"
            .parse::<DocumentMut>()
            .unwrap();
        sync_document(&mut dst, &src);
        assert_eq!(
            dst.to_string(),
            "# test comment\n\nkey1 = \"new_value\"\nkey2 = \"value3\"\n[section]\nkey = \"test\"\n#test comment"
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_sync_document_3() {
        let mut dst: DocumentMut = r#"
# test comment

key1 = "value1"
key2 = "value2"

[section]
test = 123
test2 = "value_2"
# test comment

[section_2]
test = 546
# test2 = "value_4"
"#
        .parse()
        .unwrap();
        #[derive(serde::Serialize)]
        struct Section {
            test: u16,
            test2: Option<String>,
        }
        #[derive(serde::Serialize)]
        struct Config {
            key1: String,
            key2: String,
            section: Section,
            section_2: Section,
        }
        let src = toml_edit::ser::to_document(&Config {
            key1: "new_value".to_string(),
            key2: "value3".to_string(),
            section: Section {
                test: 345,
                test2: None,
            },
            section_2: Section {
                test: 161,
                test2: Some("value_2".to_string()),
            },
        })
        .expect("config should always serialize");
        sync_document(&mut dst, &src);
        assert_eq!(
            dst.to_string(),
            r#"
# test comment

key1 = "new_value"
key2 = "value3"

[section]
test = 345
test2 = "value_2"
# test comment

[section_2]
test = 161
test2 = "value_2"
# test2 = "value_4"
"#
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_sync_document_array_0() {
        let mut dst: DocumentMut = r#"
[[sections]]
test = 123
[[sections]]
test = 546
test2 = "value_4""#
            .parse()
            .unwrap();
        dst.as_table_mut()
            .get_mut("sections")
            .unwrap()
            .as_array_of_tables_mut()
            .unwrap()
            .get_mut(0)
            .unwrap()
            .insert("test", toml_edit::value(345));
        assert_eq!(
            dst.to_string(),
            r#"
[[sections]]
test = 345
[[sections]]
test = 546
test2 = "value_4"
"#
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_sync_document_array_1() {
        let mut dst: DocumentMut = r#"
# Comment
[[sections]]
test = 123
test3 = "value_2"
[[sections]]
test = 546
test2 = "value_4"
test3 = "value_8"
[[sections]]
test = 122
# Comment
"#
        .parse()
        .unwrap();
        let src: DocumentMut = r#"
[[sections]]
test = 123
test2 = "value_1"
[[sections]]
test = 546
test3 = "value_5"
"#
        .parse()
        .unwrap();
        sync_array(
            dst.get_mut("sections").unwrap(),
            src.get("sections").unwrap(),
            |a| a.get("test").unwrap().as_integer().unwrap(),
        )
        .unwrap();

        assert_eq!(
            dst.to_string(),
            r#"
# Comment
[[sections]]
test = 123
test3 = "value_2"
test2 = "value_1"
[[sections]]
test = 546
test2 = "value_4"
test3 = "value_5"
# Comment
"#
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_sync_document_array_2() {
        let mut dst: DocumentMut = r#"
test = 12

[[sections]]
id = 123
[[sections]]
id = 546
test2 = "value_4"
[[sections]]
id = 122
test2 = "value_8"
"#
        .parse()
        .unwrap();
        #[derive(serde::Serialize)]
        struct Section {
            id: u16,
            test2: Option<String>,
        }
        #[derive(serde::Serialize)]
        struct Config {
            test: u16,
            sections: Vec<Section>,
        }
        let src = toml_edit::ser::to_document(&Config {
            test: 1,
            sections: vec![
                Section {
                    id: 123,
                    test2: None,
                },
                Section {
                    id: 122,
                    test2: Some("value_2".to_string()),
                },
            ],
        })
        .expect("config should always serialize");
        sync_document(&mut dst, &src);
        sync_array(
            dst.get_mut("sections").unwrap(),
            src.get("sections").unwrap(),
            |a| a.get("id").unwrap().as_integer().unwrap(),
        )
        .unwrap();
        assert_eq!(
            dst.to_string(),
            r#"
test = 1

[[sections]]
id = 123
[[sections]]
id = 122
test2 = "value_2"
"#
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_sync_document_array_3() {
        let mut dst: DocumentMut = r#"
# Comment
test = "test" # test value
sections = [ { test = 123, test3 = "value_2",    }, { test = 546, test2 = "value_4", test3= "value_8"   }, { test = 122 } ]
# Comment
"#
        .parse()
        .unwrap();
        let src: DocumentMut = r#"
[[sections]]
test = 123
test2 = "value_1"
[[sections]]
test = 546
test3 = "value_5"
"#
        .parse()
        .unwrap();
        sync_array(
            dst.get_mut("sections").unwrap(),
            src.get("sections").unwrap(),
            |a| a.get("test").unwrap().as_integer().unwrap(),
        )
        .unwrap();

        assert_eq!(
            dst.to_string(),
            r#"
# Comment
test = "test" # test value
sections = [ { test = 123, test3 = "value_2", test2 = "value_1",    }, { test = 546, test2 = "value_4", test3= "value_5"}]
# Comment
"#
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_sync_document_array_4() {
        let mut dst: DocumentMut = r#"
# Comment
test = "test" # test value
sections = [ { test = 123, test3 = "value_2",    }, { test = 546, test2 = "value_4", test3= "value_8"   }, { test = 122 } ]
# Comment
"#
        .parse()
        .unwrap();
        let src: DocumentMut = r#"
sections = [ { test = 123, test2 = "value_1" }, { test = 546, test3= "value_5" } ]
"#
        .parse()
        .unwrap();
        sync_array(
            dst.get_mut("sections").unwrap(),
            src.get("sections").unwrap(),
            |a| a.get("test").unwrap().as_integer().unwrap(),
        )
        .unwrap();

        assert_eq!(
            dst.to_string(),
            r#"
# Comment
test = "test" # test value
sections = [ { test = 123, test3 = "value_2", test2 = "value_1" ,    }, { test = 546, test2 = "value_4", test3= "value_5" }]
# Comment
"#
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_sync_document_map_0() {
        let mut dst: DocumentMut = r#"
sources = { old = { name = "old", kind = "basic", extra = "remove" } }
"#
        .parse()
        .unwrap();
        let src: DocumentMut = r#"
sources = { new = { name = "new", kind = "simple" } }
"#
        .parse()
        .unwrap();
        sync_document(&mut dst, &src);
        assert_eq!(
            dst.to_string(),
            r#"
sources = { new = { name = "new", kind = "simple" } }
"#
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_sync_document_map_1() {
        let mut dst: DocumentMut = r#"
# Comment
sources = { alpha = { name = "alpha", kind = "old" } }
# sources = { alpha = { name = "alpha", kind = "old" } }
"#
        .parse()
        .unwrap();
        let src: DocumentMut = r#"
sources = { beta = { name = "beta", kind = "updated", extra = "added" } }
"#
        .parse()
        .unwrap();
        sync_document(&mut dst, &src);
        assert_eq!(
            dst.to_string(),
            r#"
# Comment
sources = { beta = { name = "beta", kind = "updated", extra = "added" } }
# sources = { alpha = { name = "alpha", kind = "old" } }
"#
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_sync_document_map_2() {
        let mut dst: DocumentMut = r#"
sources = { old_1 =    { name = "old_1", kind = "basic" },old_2 = { name =    "old_2", kind = "basic", extra = "remove" } }
# comment
"#
        .parse()
        .unwrap();
        let src: DocumentMut = r#"
sources = { new_1 = { name = "new_1", kind = "simple" }, new_2 = { name = "new_2", kind = "simple", extra = "added" } }
"#
        .parse()
        .unwrap();
        sync_document(&mut dst, &src);
        assert_eq!(
            dst.to_string(),
            r#"
sources = { new_1 = { name = "new_1", kind = "simple" }, new_2 = { name = "new_2", kind = "simple", extra = "added" } }
# comment
"#
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_sync_document_map_3() {
        let mut dst: DocumentMut = r#"
# Comment
sources = { first = { name = "first", kind = "old" }, second = { name = "second", kind = "old", extra = "remove" } }
# sources = { first = { name = "first", kind = "old" }, second = { name = "second", kind = "old", extra = "remove" } }
"#
        .parse()
        .unwrap();
        let src: DocumentMut = r#"
sources = { }
"#
        .parse()
        .unwrap();
        sync_document(&mut dst, &src);
        assert_eq!(
            dst.to_string(),
            r#"
# Comment
sources = { }
# sources = { first = { name = "first", kind = "old" }, second = { name = "second", kind = "old", extra = "remove" } }
"#
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_sync_document_map_4() {
        let mut dst: DocumentMut = r#"
sources = { }
"#
        .parse()
        .unwrap();
        let src: DocumentMut = r#"
sources = { one = { name = "one", kind = "updated" }, two = { name = "two", kind = "updated", extra = "added" } }
"#
        .parse()
        .unwrap();
        sync_document(&mut dst, &src);
        assert_eq!(
            dst.to_string(),
            r#"
sources = { one = { name = "one", kind = "updated" }, two = { name = "two", kind = "updated", extra = "added" } }
"#
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_sync_document_map_5() {
        let mut dst: DocumentMut = r#"
sources = { }
"#
        .parse()
        .unwrap();
        #[derive(serde::Serialize)]
        struct SourceItem {
            name: String,
            kind: String,
            extra: Option<String>,
        }
        #[derive(serde::Serialize)]
        struct Config {
            sources: std::collections::HashMap<String, SourceItem>,
        }
        let src = toml_edit::ser::to_document(&Config {
            sources: [
                (
                    "one".to_string(),
                    SourceItem {
                        name: "one ".to_string(),
                        kind: "updated".to_string(),
                        extra: None,
                    },
                ),
                (
                    "two".to_string(),
                    SourceItem {
                        name: "two".to_string(),
                        kind: "updated".to_string(),
                        extra: Some("added  ".to_string()),
                    },
                ),
            ]
            .into_iter()
            .collect(),
        })
        .expect("config should always serialize");
        sync_document(&mut dst, &src);
        assert_eq!(
            dst.to_string(),
            r#"
sources = { one = { name = "one ", kind = "updated" }, two = { name = "two", kind = "updated", extra = "added  " } }
"#
        );
    }
}
