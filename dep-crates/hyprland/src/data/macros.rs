macro_rules! impl_on {
    ($name:ident) => {
        impl HyprData for $name {
            fn get() -> $crate::Result<Self> {
                Self::instance_get(crate::default_instance()?)
            }
            fn instance_get(instance: &crate::instance::Instance) -> $crate::Result<Self> {
                let data = instance.write_to_socket(command!(JSON, "{}", DataCommands::$name))?;
                let deserialized: $name = serde_json::from_str(&data)?;
                Ok(deserialized)
            }
            #[cfg(any(feature = "async-lite", feature = "tokio"))]
            async fn get_async() -> $crate::Result<Self> {
                Self::instance_get_async(crate::default_instance()?).await
            }
            #[cfg(any(feature = "async-lite", feature = "tokio"))]
            async fn instance_get_async(
                instance: &crate::instance::Instance,
            ) -> $crate::Result<Self> {
                let data = instance
                    .write_to_socket_async(command!(JSON, "{}", DataCommands::$name))
                    .await?;
                let deserialized: $name = serde_json::from_str(&data)?;
                Ok(deserialized)
            }
        }
    };
}

macro_rules! implement_iterators {
    (
        vector,
        name: $name:ident,
        iterated_field: $iterated_field:tt,
        holding_type: $holding_type:ty,
    ) => {
        impl $name {
            paste!(
                #[doc = "Creates the iterator by references of `" $name "`."]
                pub fn iter(&self) -> std::slice::Iter<'_, $holding_type> {
                    self.0.iter()
                }

                #[doc = "Creates the iterator by mutable references of " $name "`."]
                pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, $holding_type> {
                    self.0.iter_mut()
                }
            );
        }

        impl IntoIterator for $name {
            type Item = $holding_type;

            type IntoIter = std::vec::IntoIter<$holding_type>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.into_iter()
            }
        }

        impl<'a> IntoIterator for &'a $name {
            type Item = &'a $holding_type;
            type IntoIter = std::slice::Iter<'a, $holding_type>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.iter()
            }
        }

        impl<'a> IntoIterator for &'a mut $name {
            type Item = &'a mut $holding_type;
            type IntoIter = std::slice::IterMut<'a, $holding_type>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.iter_mut()
            }
        }
    };

    (
        table,
        name: $name:ident,
        iterated_field: $iterated_field:tt,
        key: $key:ty,
        value: $value:ty,
    ) => {
        impl $name {
            paste!(
                #[doc = "Creates the iterator of map by references of " $name]
                pub fn iter(&self) -> std::collections::hash_map::Iter<'_, $key, $value> {
                    self.$iterated_field.iter()
                }

                #[doc = "Creates the iterator of map by mutable references of `" $name "`."]
                pub fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<'_, $key, $value> {
                    self.$iterated_field.iter_mut()
                }

                #[doc = "Creates the consuming iterator by keys with type `" $key "` of `" $name "`."]
                pub fn into_keys(self) -> std::collections::hash_map::IntoKeys<$key, $value> {
                    self.$iterated_field.into_keys()
                }

                #[doc = "Creates the consuming iterator by values of `" $name "`."]
                pub fn into_values(self) -> std::collections::hash_map::IntoValues<$key, $value> {
                    self.$iterated_field.into_values()
                }
            );
        }

        impl IntoIterator for $name {
            type Item = ($key, $value);
            type IntoIter = std::collections::hash_map::IntoIter<$key, $value>;

            fn into_iter(self) -> Self::IntoIter {
                self.$iterated_field.into_iter()
            }
        }

        impl<'a> IntoIterator for &'a $name {
            type Item = (&'a $key, &'a $value);
            type IntoIter = std::collections::hash_map::Iter<'a, $key, $value>;

            fn into_iter(self) -> Self::IntoIter {
                self.$iterated_field.iter()
            }
        }

        impl<'a> IntoIterator for &'a mut $name {
            type Item = (&'a $key, &'a mut $value);
            type IntoIter = std::collections::hash_map::IterMut<'a, $key, $value>;

            fn into_iter(self) -> Self::IntoIter {
                self.$iterated_field.iter_mut()
            }
        }
    }
}

macro_rules! create_data_struct {
    (
        vector,
        name: $name:ident,
        command: $cmd_kind:path,
        holding_type: $holding_type:ty,
        doc: $doc:literal
    ) => {
        #[doc = $doc]
        #[derive(Debug, Clone)]
        pub struct $name(Vec<$holding_type>);

        implement_iterators!(
            vector,
            name: $name,
            iterated_field: 0,
            holding_type: $holding_type,
        );

        impl HyprData for $name {
            fn get() -> $crate::Result<Self> {
                Self::instance_get(crate::default_instance()?)
            }
            fn instance_get(instance: &crate::instance::Instance) -> $crate::Result<Self> {
                let data = instance.write_to_socket(command!(JSON, "{}", $cmd_kind))?;
                let deserialized: Vec<$holding_type> = serde_json::from_str(&data)?;
                Ok(Self(deserialized))
            }
            #[cfg(any(feature = "async-lite", feature = "tokio"))]
            async fn get_async() -> $crate::Result<Self> {
                Self::instance_get_async(crate::default_instance()?).await
            }
            #[cfg(any(feature = "async-lite", feature = "tokio"))]
            async fn instance_get_async(instance: &crate::instance::Instance) -> $crate::Result<Self> {
                let data = instance.write_to_socket_async(command!(JSON, "{}", $cmd_kind)).await?;
                let deserialized: Vec<$holding_type> = serde_json::from_str(&data)?;
                Ok(Self(deserialized))
            }
        }

        impl HyprDataVec<$holding_type> for $name {
            fn to_vec(self) -> Vec<$holding_type> {
                self.0
            }
        }
    };

    (
        table,
        name: $name:ident,
        command: $cmd_kind:path,
        key: $key:ty,
        value: $value:ty,
        doc: $doc:literal
    ) => {
        #[doc = $doc]
        #[derive(Debug)]
        pub struct $name(HashMap<$key, $value>);

        implement_iterators!(
            table,
            name: $name,
            iterated_field: 0,
            key: $key,
            value: $value,
        );

        impl HyprData for $name {
            fn get() -> $crate::Result<Self> {
                Self::instance_get(crate::default_instance()?)
            }
            fn instance_get(instance: &crate::instance::Instance) -> $crate::Result<Self> {
                let data = instance.write_to_socket(command!(JSON, "{}", $cmd_kind))?;
                let deserialized: HashMap<$key, $value> = serde_json::from_str(&data)?;
                Ok(Self(deserialized))
            }
            #[cfg(any(feature = "async-lite", feature = "tokio"))]
            async fn get_async() -> $crate::Result<Self> {
                Self::instance_get_async(crate::default_instance()?).await
            }
            #[cfg(any(feature = "async-lite", feature = "tokio"))]
            async fn instance_get_async(instance: &crate::instance::Instance) -> $crate::Result<Self> {
                let data = instance.write_to_socket_async(command!(JSON, "{}", $cmd_kind)).await?;
                let deserialized: HashMap<$key, $value> = serde_json::from_str(&data)?;
                Ok(Self(deserialized))
            }
        }
    };
}
