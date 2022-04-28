use crate::{Annotations, RustType, ToSourceFilePart, Visibility};
use std::fmt::Formatter;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConstantEntity {
    annotations: Annotations,
    constant_type: ConstantType,
    visibility: Visibility,
    name: String,
    rust_type: RustType,
    value: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ConstantType {
    Const,
    Static,
}

impl std::fmt::Display for ConstantType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ConstantType::Const => "const",
            ConstantType::Static => "static",
        })
    }
}

impl ToSourceFilePart for ConstantEntity {
    fn format_source_file_part(&self) -> String {
        self.format()
    }
}

impl ConstantEntity {
    pub fn new(
        annotations: Annotations,
        constant_type: ConstantType,
        visibility: Visibility,
        name: impl Into<String>,
        rust_type: RustType,
        value: impl Into<String>,
    ) -> Self {
        Self {
            annotations,
            constant_type,
            visibility,
            name: name.into(),
            rust_type,
            value: value.into(),
        }
    }

    pub fn format(&self) -> String {
        format!(
            "{}{}{} {}: {} = {};\n",
            self.annotations.format(),
            self.visibility,
            self.constant_type,
            self.name,
            self.rust_type.format(),
            self.value
        )
    }
}
