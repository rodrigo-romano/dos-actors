use std::fmt::Display;

use crate::Names;

pub struct GetIO<'a> {
    kind: String,
    variants: &'a Names,
}
impl<'a> GetIO<'a> {
    pub fn new<S: Into<String>>(kind: S, variants: &'a Names) -> Self {
        Self {
            kind: kind.into(),
            variants,
        }
    }
}
impl<'a> Display for GetIO<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let arms = self
            .variants
            .iter()
            .map(|name| {
                format!(
                    r#""{0}" => Ok(Box::new(SplitFem::<{1}>::new()))"#,
                    name,
                    name.variant()
                )
            })
            .collect::<Vec<String>>()
            .join(",\n");
        write!(
            f,
            "
        impl TryFrom<String> for Box<dyn Get{io}> {{
            type Error = FemError;
            fn try_from(value: String) -> std::result::Result<Self, Self::Error> {{
                match value.as_str() {{
                    {arms},
                    _ => Err(FemError::Convert(value)),
                }}
            }}
         }}
        ",
            io = self.kind,
            arms = arms
        )?;

        let variants = self
            .variants
            .iter()
            .map(|name| {
                format!(
                    r#"
        if let Some(x) = SplitFem::<{1}>::get_{0}(self) {{
            return x.serialize(s);
        }}
        "#,
                    self.kind.to_lowercase(),
                    name.variant()
                )
            })
            .collect::<Vec<String>>()
            .join("\n");
        write!(
            f,
            r##"
#[cfg(feature = "serde")]
impl serde::Serialize for Box<dyn Get{io}> {{
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {{
        {variants}
        Err(serde::ser::Error::custom(
            "failed to downcast `SplitFem<U>` with `U` as actors {io}puts",
        ))
    }}
}}
    "##,
            io = self.kind,
            variants = variants
        )?;

        let variants = self
            .variants
            .iter()
            .map(|name| {
                format!(
                    r#"
    "{0}" => Ok(Box::new(SplitFem::<{0}>::ranged(
        deser.range.clone(),
    )))
    "#,
                    name.variant()
                )
            })
            .collect::<Vec<String>>()
            .join(",\n");
        write!(
            f,
            r##"
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Box<dyn Get{io}> {{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {{
        let deser = super::SplitFemErased::deserialize(deserializer)?;
        match deser.kind.as_str() {{
            {variants},
            _ => Err(serde::de::Error::custom(
                "failed to deserialize into `SplitFem<U>` with `U` as actors {io}puts",
            ))
        }}
    }}
}}
    "##,
            io = self.kind,
            variants = variants
        )?;

        Ok(())
    }
}
