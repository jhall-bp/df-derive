#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NestedNamePolicy {
    Field,
    Flatten,
    Prefix(String),
}

impl NestedNamePolicy {
    pub const fn requires_unique_name_validation(&self) -> bool {
        !matches!(self, Self::Field)
    }
}

pub fn column_name_for_ident(ident: &syn::Ident) -> String {
    let name = ident.to_string();
    name.strip_prefix("r#").unwrap_or(&name).to_owned()
}
