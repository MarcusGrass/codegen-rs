use std::fmt::Write;
use crate::structures::generics::Generics;
use crate::structures::visibility::Visibility;
use crate::structures::{Annotations, ComponentSignature, Derives};
use crate::{NamedComponentSignature, RustType};

#[derive(Debug)]
pub struct StructEntity {
    annotations: Annotations,
    derives: Derives,
    visibility: Visibility,
    name: String,
    kind: StructKind,
}

#[derive(Debug)]
pub enum StructKind {
    Fields(Vec<Field>),
    Container(Vec<(Visibility, RustType)>),
}

impl StructEntity {
    pub fn new(
        annotations: Annotations,
        derives: Derives,
        visibility: Visibility,
        name: impl Into<String>,
        kind: StructKind,
    ) -> Self {
        Self {
            annotations,
            derives,
            visibility,
            name: name.into(),
            kind,
        }
    }

    pub fn format(&self) -> String {
        match &self.kind {
            StructKind::Fields(fields) => {
                let mut union = Generics::default();
                for generics in
                    fields
                        .iter()
                        .map(|f| match &f.named_component_signature.component_signature {
                            ComponentSignature::Signature(s) => s.generics.clone(),
                            ComponentSignature::Generic(g) => Generics::multiple(vec![g.clone()]),
                        })
                {
                    union = union.union(&generics);
                }
                let diamond_typed = union.format_diamond_typed();
                let bounds = union.format_where_clause();
                let mut base = format!(
                    "{}{}{}struct {}{diamond_typed} {bounds}{{\n",
                    self.annotations.format(),
                    self.derives.format(),
                    self.visibility,
                    self.name
                );
                for field in fields {
                    base.push_str(&field.format_line());
                }
                base.push_str("}\n");
                base
            }
            StructKind::Container(c) => {
                let mut contained = String::new();
                for (ind, (v, r)) in c.iter().enumerate() {
                    if ind != 0 {
                        contained.push_str(", ");
                    }
                    let _ = contained.write_fmt(format_args!("{}{}", v.format(), r.format()));
                }
                format!(
                    "{}{}{}struct {}({});\n",
                    self.annotations.format(),
                    self.derives.format(),
                    self.visibility,
                    self.name,
                    contained
                )
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    visibility: Visibility,
    pub named_component_signature: NamedComponentSignature,
}

impl Field {
    pub fn new(visibility: Visibility, named_component_signature: NamedComponentSignature) -> Self {
        Self {
            visibility,
            named_component_signature,
        }
    }

    pub fn format_line(&self) -> String {
        format!(
            "{}{}: {},\n",
            self.visibility,
            self.named_component_signature.name,
            self.named_component_signature.component_signature
        )
    }
}
