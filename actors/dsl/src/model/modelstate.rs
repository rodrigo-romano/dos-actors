use std::fmt::Display;

use syn::Ident;

use super::keyparam::Param;

/// State of the model
///
/// This is state that the model will be into when handed over to the main scope
#[derive(Default, Debug, Clone)]
pub enum ModelState {
    Ready,
    Running,
    #[default]
    Completed,
}

impl Display for ModelState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelState::Ready => write!(f, "ready"),
            ModelState::Running => write!(f, "running"),
            ModelState::Completed => write!(f, "completed"),
        }
    }
}

impl TryFrom<Ident> for ModelState {
    type Error = syn::Error;
    fn try_from(value: Ident) -> Result<Self, Self::Error> {
        match value.to_string().as_str() {
            "ready" => Ok(Self::Ready),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            _ => Err(syn::Error::new(
                value.span(),
                format!(r#"expected state "ready", "running" or "completed", found {value}"#),
            )),
        }
    }
}

impl TryFrom<&Param> for ModelState {
    type Error = syn::Error;

    fn try_from(value: &Param) -> Result<Self, Self::Error> {
        Ok(Ident::try_from(value)?.try_into()?)
    }
}
