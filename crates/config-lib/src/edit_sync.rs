use anyhow::{Context, bail};
use toml_edit::{DocumentMut, Item, Table, TableLike, Value};
use tracing::{error, trace};

/// Convert data into a pretty DocumentMut
pub fn to_pretty_document<T: serde::Serialize + ?Sized>(data: &T) -> DocumentMut {
    toml_edit::ser::to_string_pretty(data)
        .ok()
        .and_then(|conf| conf.parse::<toml_edit::DocumentMut>().ok())
        .unwrap_or_else(toml_edit::DocumentMut::new)
}

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
    let mut src_items: Vec<_> = src.iter().collect();
    src_items.sort_by(|(left, _), (right, _)| left.cmp(right));
    let mut dst_keys: Vec<_> = dst.iter().map(|(key, _)| key.to_string()).collect();

    for (key, src_item) in src_items {
        // update value of key
        if dst.contains_key(key) {
            let dst_item = dst.get_mut(key).expect("key was present");
            sync_item(dst_item, src_item);
            dst_keys.retain(|dst_key| dst_key != key);
            // replace first key value (keeps comments, etc)
        } else if let Some(dst_key) = dst_keys.pop() {
            let (_, dst_item) = dst
                .get_key_value_mut(&dst_key)
                .expect("key was collected from destination");
            // *dst_item = src_item.clone();
            sync_item(dst_item, src_item);
            let dst_item = dst_item.clone();
            dst.remove(&dst_key);
            dst.insert(key, dst_item);
            // no remaining items in dst
        } else {
            let dst_item = dst.entry(key).or_insert(Item::None);
            sync_item(dst_item, src_item);
        }
    }

    // remove old keys
    for key in dst_keys {
        dst.remove(&key);
    }
}

fn sync_item(d: &mut Item, s: &Item) {
    let d_name = d.type_name();
    let s_name = s.type_name();
    match (d, s) {
        (Item::Table(dst), Item::Table(src)) => {
            #[cfg(debug_assertions)]
            tracing::trace!(
                "Syncing t2t: dst[{}]:<{}>, src[{}]:<{}>",
                d_name,
                dst,
                s_name,
                src
            );
            sync(dst, src);
        }
        (Item::Table(dst), Item::Value(Value::InlineTable(src))) => {
            #[cfg(debug_assertions)]
            tracing::trace!(
                "Syncing t2it: dst[{}]:<{}>, src[{}]:<{}>",
                d_name,
                dst,
                s_name,
                src
            );
            sync(dst, src);
        }
        (Item::Value(Value::InlineTable(dst)), Item::Table(src)) => {
            #[cfg(debug_assertions)]
            #[cfg(debug_assertions)]
            tracing::trace!(
                "Syncing it2t: dst[{}]:<{}>, src[{}]:<{}>",
                d_name,
                dst,
                s_name,
                src
            );
            sync(dst, src);
        }
        (Item::Value(Value::InlineTable(dst)), Item::Value(Value::InlineTable(src))) => {
            #[cfg(debug_assertions)]
            tracing::trace!(
                "Syncing it2it: dst[{}]:<{}>, src[{}]:<{}>",
                d_name,
                dst,
                s_name,
                src
            );
            sync(dst, src);
        }
        // For anything else, fall back to replacement.
        (dst, src) if let Item::None = dst => {
            #[cfg(debug_assertions)]
            tracing::trace!(
                "Replacing item: dst = None, src[{}]:<{}>",
                src.type_name(),
                format!("{}", src).replace("\n", "\\n")
            );
            *dst = src.clone();
        }
        // For scalar values, just replace the value in place.
        (Item::Value(dst), Item::Value(src)) => {
            #[cfg(debug_assertions)]
            tracing::trace!(
                "Syncing value: dst[{}]:<{}>, src[{}]:<{}>",
                dst.type_name(),
                format!("{}", dst).replace("\n", "\\n"),
                src.type_name(),
                format!("{}", src).replace("\n", "\\n")
            );
            // only replace the value, keep prefix and suffix text
            dst.with_value_strict(src);
        }
        (Item::ArrayOfTables(_), _) | (_, Item::ArrayOfTables(_)) => {
            // skipped, must be manually done using sync_array
            trace!("array of tables detected");
        }
        (dst, src) => {
            error!(
                "not syncing different types: [{}] -> [{}]",
                src.type_name(),
                dst.type_name()
            );
        }
    }
}

trait ReplaceValue {
    fn with_value_strict(&mut self, new: &Value);
    /// Used for comparisun, so no float
    fn get_value(&self) -> Option<ValueRepr>;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ValueRepr {
    String(String),
    Integer(i64),
    Boolean(bool),
    Datetime(toml_edit::Datetime),
}

macro_rules! replace_formatted_value {
    ($dst:expr, $value:expr) => {{
        let decoration = $dst.decor().clone();
        *$dst = toml_edit::Formatted::new($value);
        *$dst.decor_mut() = decoration;
    }};
}

impl ReplaceValue for Value {
    fn get_value(&self) -> Option<ValueRepr> {
        match self {
            Value::String(f) => Some(ValueRepr::String(f.value().clone())),
            Value::Integer(f) => Some(ValueRepr::Integer(*f.value())),
            Value::Boolean(f) => Some(ValueRepr::Boolean(*f.value())),
            Value::Datetime(f) => Some(ValueRepr::Datetime(*f.value())),
            Value::Array(_) | Value::InlineTable(_) | Value::Float(_) => None,
        }
    }

    fn with_value_strict(&mut self, new: &Value) {
        match (self, new) {
            (Value::Integer(ds), Value::Integer(sr)) => {
                replace_formatted_value!(ds, *sr.value());
            }
            (Value::String(ds), Value::String(sr)) => {
                replace_formatted_value!(ds, sr.value().clone());
            }
            (Value::Float(ds), Value::Float(sr)) => {
                replace_formatted_value!(ds, sr.value().clone());
            }
            (Value::Boolean(ds), Value::Boolean(sr)) => {
                replace_formatted_value!(ds, sr.value().clone());
            }
            (Value::Datetime(ds), Value::Datetime(sr)) => {
                replace_formatted_value!(ds, sr.value().clone());
            }
            (Value::Array(ds), Value::Array(sr)) => {
                if ds.is_empty() {
                    let decoration = ds.decor().clone();
                    *ds = toml_edit::Array::from_iter(sr);
                    *ds.decor_mut() = decoration;
                } else if sr.is_empty() {
                    ds.clear();
                } else {
                    let test = sr.get(0).expect("must have 1. element");
                    match test {
                        Value::InlineTable(_) => {
                            trace!("array of inline tables detected");
                            return;
                        }
                        _ => {
                            ds.set_trailing_comma(false);

                            // valid indexes in array
                            let sr_len = sr.len();
                            for (idx, src_el) in sr.into_iter().enumerate() {
                                if let Value::InlineTable(_) = src_el {
                                    trace!("array of inline tables detected");
                                    break;
                                }

                                // replace value if prev existed
                                let pos =
                                    ds.iter().position(|i| i.get_value() == src_el.get_value());
                                if let Some(d_idx) = pos {
                                    if d_idx == idx {
                                        trace!("already at correct location");
                                        continue;
                                    }
                                    trace!("found existing in {d_idx} (in src: {idx})");
                                    // already exists somewhere, switch positions
                                    if ds.get_mut(idx).is_some() {
                                        let found_clone =
                                            ds.get(d_idx).expect("just got from position").clone();
                                        let a = ds.get_mut(idx).expect("just checked it is some");
                                        let tmp = a.clone();
                                        // a.with_value_strict(&src_el);
                                        *a = found_clone;
                                        let old =
                                            ds.get_mut(d_idx).expect("just got from position");
                                        *old = tmp;
                                    } else {
                                        trace!("dst to short");
                                        ds.insert(idx, src_el);
                                    }
                                } else {
                                    if ds.len() <= idx {
                                        trace!("dst to short, pushing back");
                                        ds.push(src_el);
                                    } else {
                                        trace!("moving element to back, replacing");
                                        let val = ds.get_mut(idx).expect("just checked it is some");
                                        let tmp = val.clone();
                                        // we full replace instead of value replace, because surrounding was moved to end
                                        // else it would be cloned
                                        *val = src_el.clone();
                                        ds.push(tmp);
                                    }
                                }
                                trace!("{}", ds);
                            }
                            // trim off elements from all elements from ds that are < sr.len()
                            while ds.len() > sr_len {
                                trace!("removing idx {}", ds.len() - 1);
                                ds.remove(ds.len() - 1);
                            }
                            // TODO maybe set suffix of last el to \n
                        }
                    }
                }
            }
            (Value::InlineTable(_), Value::InlineTable(_)) => {
                panic!("should be handled");
            }
            (a, b) => {
                error!(
                    "not syncing different types: [{}] -> [{}]",
                    b.type_name(),
                    a.type_name()
                );
            }
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
mod sync {
    use super::*;
    use toml_edit::DocumentMut;

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_sync_document_0() {
        let mut dst: DocumentMut = r#"# test comment
[section]
# some value
key1 = "value1"
# end
"#
        .parse()
        .unwrap();
        let src = r#"# test comment
[section]
key1 = "new_value"
key2 = "value2"
"#
        .parse::<DocumentMut>()
        .unwrap();
        sync_document(&mut dst, &src);

        println!("raw:\n{}", dst.to_string());
        assert_eq!(
            dst.to_string(),
            r#"# test comment
[section]
# some value
key1 = "new_value"
key2 = "value2"
# end
"#
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_sync_document_1() {
        let mut dst: DocumentMut = r#"# test comment
[section]
# some value
key1 = "value1" # test
# middle
key2 =43.34
# key3 = "test"
"#
        .parse()
        .unwrap();
        let src = r#"# test comment
[section]
key1 = "new_value"
key2 = 34.234
"#
        .parse::<DocumentMut>()
        .unwrap();
        sync_document(&mut dst, &src);

        println!("raw:\n{}", dst.to_string());
        assert_eq!(
            dst.to_string(),
            r#"# test comment
[section]
# some value
key1 = "new_value" # test
# middle
key2 =34.234
# key3 = "test"
"#
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_sync_document_2() {
        let mut dst: DocumentMut = r#"# test comment
key1 = false
# middle
key2=[12,34,54]
key3 = []
key4 =[ 233, 454]
key5 = [
  233, # test
  111,
  # 44,
  454
]
    "#
        .parse()
        .unwrap();
        let src = r#"key1 = true
key2 = [ 1 , 2 , 3 ]
key3 = [ 3]
key4= [   ]
key5 = [
  111,
  454,
  233
]
[section]
keyn = "new_value"
    "#
        .parse::<DocumentMut>()
        .unwrap();
        sync_document(&mut dst, &src);

        println!("raw:\n{}", dst.to_string());
        assert_eq!(
            dst.to_string(),
            r#"# test comment
key1 = true
# middle
key2=[ 1 , 2 , 3 ]
key3 = [ 3]
key4 =[]
key5 = [ # test
  111,
  # 44,
  454
,
  233]
[section]
keyn = "new_value"
    "#
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_sync_document_3() {
        let mut dst: DocumentMut = r#"# test comment
arr1 = [ ]
arr2 = [ 100 ]
arr3 = [ 100 ]
arr4 = [
  # this is 100
  100,

  # this is 50
  50 # wow
]
arr5 = [
  # this is 100
  100,

  # this is 50
  50 # wow
]
arr6 = [
  # this is 100
  100,

  # this is 50
  50,
]
arr7 = [
  # this is 100
  100,

  # this is 50
  50,
]
"#
        .parse()
        .unwrap();
        let src = r#"#new line
arr1= [22]
arr2= []
arr3= [300, 600, 100]
arr4= [
  300,
  100,
  600
]
arr5= [
  300,
  100,
  600,
]
arr6= [
  300,
  100,
  600
]
arr7= [
  300,
  100,
  600,
]
"#
        .parse::<DocumentMut>()
        .unwrap();
        sync_document(&mut dst, &src);

        println!("raw:\n{}", dst.to_string());
        assert_eq!(
            dst.to_string(),
            r#"# test comment
arr1 = [22]
arr2 = []
arr3 = [300, 600, 100 ]
arr4 = [
  300,
  # this is 100
  100,
  600
]
arr5 = [
  300,
  # this is 100
  100,
  600]
arr6 = [
  300,
  # this is 100
  100,
  600

]
arr7 = [
  300,
  # this is 100
  100,
  600
]
"#
        );
    }
}

#[allow(clippy::needless_raw_string_hashes)]
// #[cfg(test)]
mod tests {
    use super::*;
    use toml_edit::DocumentMut;

    // #[test_log::test]
    // #[test_log(default_log_filter = "trace")]
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

    // #[test_log::test]
    // #[test_log(default_log_filter = "trace")]
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
        let src = to_pretty_document(&Config {
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
        });
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

    // #[test_log::test]
    // #[test_log(default_log_filter = "trace")]
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

    // #[test_log::test]
    // #[test_log(default_log_filter = "trace")]
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

    // #[test_log::test]
    // #[test_log(default_log_filter = "trace")]
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
        .expect("must succeed");
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

    // #[test_log::test]
    // #[test_log(default_log_filter = "trace")]
    fn test_sync_document_array_2_pretty() {
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
        let src = to_pretty_document(&Config {
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
        });
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

    // #[test_log::test]
    // #[test_log(default_log_filter = "trace")]
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

    // #[test_log::test]
    // #[test_log(default_log_filter = "trace")]
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

    // #[test_log::test]
    // #[test_log(default_log_filter = "trace")]
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

    // #[test_log::test]
    // #[test_log(default_log_filter = "trace")]
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

    // #[test_log::test]
    // #[test_log(default_log_filter = "trace")]
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

    // #[test_log::test]
    // #[test_log(default_log_filter = "trace")]
    fn test_sync_document_map_3() {
        let mut dst: DocumentMut = r#"
# Comment
sources = { first = { name = "first", kind = "old" }, second = { name = "second", kind = "old", extra = "remove" } }
# sources = { first = { name = "OLD", kind = "old" }, second = { name = "second", kind = "old", extra = "remove" } }
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
sources = {}
# sources = { first = { name = "OLD", kind = "old" }, second = { name = "second", kind = "old", extra = "remove" } }
"#
        );
    }

    // #[test_log::test]
    // #[test_log(default_log_filter = "trace")]
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
sources = { one = { name = "one", kind = "updated" }, two = { name = "two", kind = "updated", extra = "added" }  }
"#
        );
    }

    // #[test_log::test]
    // #[test_log(default_log_filter = "trace")]
    fn test_sync_document_map_5() {
        let mut dst: DocumentMut = r#"
    sources = { }"#
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
        .expect("must not fail");
        sync_document(&mut dst, &src);
        assert_eq!(
            dst.to_string(),
            r#"
    sources = { one = { name = "one ", kind = "updated" }, two = { name = "two", kind = "updated", extra = "added  " }  }
"#
        );
    }

    // #[test_log::test]
    // #[test_log(default_log_filter = "trace")]
    fn test_sync_document_map_5_pretty() {
        let mut dst: DocumentMut = r#"
   sources = { }"#
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
        let src = to_pretty_document(&Config {
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
        });
        sync_document(&mut dst, &src);
        assert_eq!(
            dst.to_string(),
            r#"
   sources = {one.name = "one ",one.kind = "updated",two.name = "two",two.kind = "updated",two.extra = "added  " }
"#
        );
    }

    // #[test_log::test]
    // #[test_log(default_log_filter = "trace")]
    fn test_sync_document_option_map_0() {
        let mut dst: DocumentMut = r#"
sources1 = {
  value1 = 1,
  value2 = 11
}
"#
        .parse()
        .unwrap();
        #[derive(serde::Serialize)]
        struct Item {
            value1: u8,
            value2: u8,
        }
        #[derive(serde::Serialize)]
        struct Config {
            sources1: Option<Item>,
            sources2: Option<Item>,
            sources3: Option<Item>,
        }

        let src = toml_edit::ser::to_document(&Config {
            sources1: None,
            sources2: Some(Item {
                value1: 2,
                value2: 12,
            }),
            sources3: None,
        })
        .expect("config should always serialize");
        sync_document(&mut dst, &src);
        assert_eq!(
            dst.to_string(),
            r#"sources2 = { value1 = 2, value2 = 12 }
"#
        );
    }

    // #[test_log::test]
    // #[test_log(default_log_filter = "trace")]
    fn test_sync_document_option_map_1_pretty() {
        let mut dst: DocumentMut = r#"
# Comment
sources1 = {
  value1 = 1,
  value2 = 11
}
"#
        .parse()
        .unwrap();
        #[derive(serde::Serialize)]
        struct Item {
            value1: u8,
            value2: u8,
        }
        #[derive(serde::Serialize)]
        struct Config {
            sources1: Option<Item>,
            sources2: Option<Item>,
            sources3: Option<Item>,
        }

        let src = to_pretty_document(&Config {
            sources1: None,
            sources2: Some(Item {
                value1: 2,
                value2: 12,
            }),
            sources3: None,
        });
        sync_document(&mut dst, &src);
        assert_eq!(
            dst.to_string(),
            r#"
# Comment
sources2 = {
  value1 = 2,
  value2 = 12
}
"#
        );
    }
}

#[allow(clippy::needless_raw_string_hashes)]
#[cfg(test)]
mod toml_edit_to_pretty {
    use std::{
        collections::{BTreeSet, HashMap, HashSet},
        str::FromStr,
    };

    use toml_edit::Document;

    use super::*;

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_0() {
        #[derive(serde::Serialize)]
        struct Item {
            value1: u8,
            value2: u8,
        }
        #[derive(serde::Serialize)]
        struct Config {
            sources1: Option<Item>,
            sources2: Option<Item>,
            sources3: Option<Item>,
        }
        let doc = to_pretty_document(&Config {
            sources1: None,
            sources2: Some(Item {
                value1: 2,
                value2: 12,
            }),
            sources3: None,
        });
        assert_eq!(
            doc.to_string(),
            r#"[sources2]
value1 = 2
value2 = 12
"#
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_1() {
        #[derive(serde::Serialize, Clone)]
        struct LItem {
            fl: f32,
            flag: u64,
        }
        #[derive(serde::Serialize)]
        struct Item {
            test: String,
            test2: Box<str>,
            fl: f64,
            flag: bool,
            flag2: bool,
            date: toml_edit::Date,
            list: Vec<&'static str>,
            list2: Vec<LItem>,
        }
        let doc = to_pretty_document(&Item {
            date: toml_edit::Datetime::from_str("1979-05-27")
                .expect("must parse")
                .date
                .expect("hust have date"),
            fl: 455.23,
            flag: true,
            flag2: false,
            test: "test string".to_string(),
            test2: "test string".to_string().into_boxed_str(),
            list: vec!["test1", "test343"],
            list2: vec![
                LItem {
                    fl: 4354.5656,
                    flag: 43456456
                };
                3
            ],
        });
        println!("{}", doc.to_string());
        assert_eq!(
            doc.to_string(),
            r#"test = "test string"
test2 = "test string"
fl = 455.23
flag = true
flag2 = false
date = 1979-05-27
list = [
    "test1",
    "test343",
]

[[list2]]
fl = 4354.5654296875
flag = 43456456

[[list2]]
fl = 4354.5654296875
flag = 43456456

[[list2]]
fl = 4354.5654296875
flag = 43456456
"#
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn pretty_complex() {
        #[derive(serde::Serialize)]
        struct Item {
            value1: u8,
            value2: u8,
        }
        #[derive(serde::Serialize)]
        struct Complex {
            test: String,
            test2: Box<str>,
            fl: f64,
            flag: bool,
            flag2: bool,
            date: toml_edit::Date,
            list: HashMap<&'static str, Item>,
        }
        #[derive(serde::Serialize)]
        struct Config {
            num: u16,
            sources1: Option<Item>,
            vec: Vec<Item>,
            vec_e: Vec<Item>,
            item: Item,
            compex: Complex,
        }
        let doc = to_pretty_document(&Config {
            sources1: Some(Item {
                value1: 43,
                value2: 54,
            }),
            item: Item {
                value1: 44,
                value2: 9,
            },
            num: 54,
            vec_e: vec![],
            vec: vec![
                Item {
                    value1: 3,
                    value2: 90,
                },
                Item {
                    value1: 130,
                    value2: 0,
                },
                Item {
                    value1: 200,
                    value2: 100,
                },
            ],
            compex: Complex {
                date: toml_edit::Datetime::from_str("1979-05-27")
                    .expect("must parse")
                    .date
                    .expect("hust have date"),
                fl: 455.23,
                flag: true,
                flag2: false,
                test: "test string".to_string(),
                test2: "test string".to_string().into_boxed_str(),
                list: HashMap::from_iter(vec![
                    (
                        "one",
                        Item {
                            value1: 1,
                            value2: 20,
                        },
                    ),
                    (
                        "two",
                        Item {
                            value1: 9,
                            value2: 100,
                        },
                    ),
                ]),
            },
        });
        println!("{}", doc.to_string());
        let base = r#"num = 54
vec_e = []

[sources1]
value1 = 43
value2 = 54

[[vec]]
value1 = 3
value2 = 90

[[vec]]
value1 = 130
value2 = 0

[[vec]]
value1 = 200
value2 = 100

[item]
value1 = 44
value2 = 9

[compex]
test = "test string"
test2 = "test string"
fl = 455.23
flag = true
flag2 = false
date = 1979-05-27

"#;

        let one = r#"[compex.list.one]
value1 = 1
value2 = 20
"#;
        let two = r#"[compex.list.two]
value1 = 9
value2 = 100
"#;
        let actual = doc.to_string();
        let expected_one_two = format!("{base}{one}\n{two}");
        let expected_two_one = format!("{base}{two}\n{one}");
        if actual != expected_one_two && actual != expected_two_one {
            panic!("actual doesnt match:\n{actual}\n{expected_one_two}\n / \n\n{expected_two_one}");
        }
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_tagged() {
        #[derive(serde::Serialize, Clone)]
        struct Item {
            flag: bool,
            num: u16,
        }
        #[derive(serde::Serialize, Clone)]
        enum Enu2 {
            Other,
            Another,
        }
        #[derive(serde::Serialize, Clone)]
        enum Enu {
            None,
            One(String),
            Two(Box<str>),
            Three(Vec<Box<str>>),
            Four(BTreeSet<u16>),
            Five(bool),
            Six(u64),
            Seven(Item),
            Eight(Enu2),
        }
        #[derive(serde::Serialize)]
        struct Config {
            e0: Enu,
            e1: Enu,
            e2: Enu,
            e3: Enu,
            e4: Enu,
            e5: Enu,
            e6: Enu,
            e7: Enu,
            e8: Enu,
            e9: Enu,
        }
        #[derive(serde::Serialize)]
        struct Main {
            config: Config,
            list: Vec<Enu>,
        }
        use Enu::*;
        let config = Config {
            e0: None,
            e1: One("one".to_string()),
            e2: Two("two".to_string().into_boxed_str()),
            e3: Three(vec!["three".to_string().into_boxed_str(); 4]),
            e4: Four(BTreeSet::from_iter(0..4)),
            e5: Five(false),
            e6: Six(5456452343),
            e7: Seven(Item {
                flag: true,
                num: 54693,
            }),
            e8: Eight(Enu2::Other),
            e9: Eight(Enu2::Another),
        };
        let doc1 = to_pretty_document(&config);
        let doc2 = to_pretty_document(&Main {
            config,
            list: vec![
                None,
                Five(false),
                Seven(Item {
                    flag: true,
                    num: 54693,
                }),
                Eight(Enu2::Other),
            ],
        });
        println!("{}\n", doc1.to_string());
        println!("{}\n", doc2.to_string());
        assert_eq!(
            doc1.to_string(),
            r#"e0 = "None"

[e1]
One = "one"

[e2]
Two = "two"

[e3]
Three = [
    "three",
    "three",
    "three",
    "three",
]

[e4]
Four = [
    0,
    1,
    2,
    3,
]

[e5]
Five = false

[e6]
Six = 5456452343

[e7.Seven]
flag = true
num = 54693

[e8]
Eight = "Other"

[e9]
Eight = "Another"
"#
        );
        assert_eq!(
            doc2.to_string(),
            r#"list = [
    "None",
    { Five = false },
    { Seven = { flag = true, num = 54693 } },
    { Eight = "Other" },
]

[config]
e0 = "None"

[config.e1]
One = "one"

[config.e2]
Two = "two"

[config.e3]
Three = [
    "three",
    "three",
    "three",
    "three",
]

[config.e4]
Four = [
    0,
    1,
    2,
    3,
]

[config.e5]
Five = false

[config.e6]
Six = 5456452343

[config.e7.Seven]
flag = true
num = 54693

[config.e8]
Eight = "Other"

[config.e9]
Eight = "Another"
"#
        );
    }

    #[test_log::test]
    #[test_log(default_log_filter = "trace")]
    fn test_untagged() {
        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
        struct Item {
            flag: bool,
            num: u16,
        }
        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
        #[serde(rename_all = "snake_case")]
        enum Enu2 {
            Other,
            Another,
        }
        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
        #[serde(untagged)]
        enum Enu {
            One(u8),
            Two(Item),
            Three(Enu2),
        }
        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
        struct Main {
            list: Vec<Enu>,
        }
        use Enu::*;
        let l = vec![
            One(3),
            Three(Enu2::Other),
            Two(Item {
                flag: false,
                num: 4554,
            }),
            Three(Enu2::Another),
        ];
        let doc = to_pretty_document(&Main { list: l.clone() });
        println!("{}\n", doc.to_string());
        assert_eq!(
            doc.to_string(),
            r#"list = [
    3,
    "other",
    { flag = false, num = 4554 },
    "another",
]
"#
        );

        let parsed = doc.to_string().parse::<DocumentMut>().expect("pust parse");
        use serde::Deserialize;
        use serde::de::IntoDeserializer;
        let deserialized = Main::deserialize(parsed.into_deserializer()).expect("must deserialize");
        println!("deserialized: {deserialized:?}");
        let matching = deserialized
            .list
            .iter()
            .zip(l.iter())
            .filter(|&(a, b)| a == b)
            .count();
        assert_eq!(matching, deserialized.list.len());
        assert_eq!(matching, l.len());
    }
}
