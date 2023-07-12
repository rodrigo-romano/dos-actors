use std::{fmt::Display, ops::Deref};

#[derive(Debug, Default)]
pub struct Name {
    name: String,
    description: Vec<String>,
}
impl Deref for Name {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.name.as_str()
    }
}
impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
impl<S: Into<String>> From<S> for Name {
    fn from(name: S) -> Self {
        Name {
            name: name.into(),
            ..Default::default()
        }
    }
}
impl From<&Name> for String {
    fn from(value: &Name) -> Self {
        value.name.clone()
    }
}
impl Name {
    pub fn push_description(&mut self, description: String) {
        self.description.push(description);
    }
    pub fn variant(&self) -> String {
        self.split("_")
            .map(|s| {
                let (first, last) = s.split_at(1);
                first.to_uppercase() + last
            })
            .collect::<String>()
    }
    /// pub enum {variant} {}
    pub fn enum_variant(&self) -> String {
        let descriptions: Vec<_> = self
            .description
            .iter()
            .map(|d| {
                format!(
                    r##"
 1. {}
            "##,
                    d
                )
            })
            .collect();
        format!(
            r##"
            #[doc = "{name}"]
            #[doc = ""]
            #[doc = "{descriptions}"]
        #[derive(Debug, ::gmt_dos_clients::interface::UID)]
        pub enum {variant} {{}}
        "##,
            name = self.name,
            descriptions = descriptions.join("\n"),
            variant = self.variant()
        )
    }
    /// impl FemIo<{variant}> for Vec<Option<{io}>>
    ///
    /// io: Inputs|Outputs
    pub fn impl_enum_variant_for_io(&self, io: &str) -> String {
        format!(
            r##"
        impl FemIo<{variant}> for Vec<Option<{io}>> {{
            fn position(&self) -> Option<usize>{{
                self.iter().filter_map(|x| x.as_ref())
                        .position(|x| if let {io}::{variant}(_) = x {{true}} else {{false}})
            }}
        }}
        "##,
            variant = self.variant(),
            io = io
        )
    }
}

#[derive(Debug, Default)]
pub struct Names(Vec<Name>);
impl FromIterator<Name> for Names {
    fn from_iter<T: IntoIterator<Item = Name>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}
impl FromIterator<String> for Names {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self(iter.into_iter().map(|x| x.into()).collect())
    }
}
impl Deref for Names {
    type Target = Vec<Name>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Display for Names {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for variant in self.iter() {
            write!(f, "{}", variant.enum_variant())?;
        }
        Ok(())
    }
}
