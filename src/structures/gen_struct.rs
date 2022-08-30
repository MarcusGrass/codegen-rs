use crate::structures::generics::Generics;
use crate::structures::visibility::Visibility;
use crate::structures::{Annotations, ComponentSignature, Derives};
use crate::NamedComponentSignature;

#[derive(Debug)]
pub struct StructEntity {
    annotations: Annotations,
    derives: Derives,
    visibility: Visibility,
    name: String,
    fields: Vec<Field>,
}

impl StructEntity {
    pub fn new(
        annotations: Annotations,
        derives: Derives,
        visibility: Visibility,
        name: impl Into<String>,
        fields: Vec<Field>,
    ) -> Self {
        Self {
            annotations,
            derives,
            visibility,
            name: name.into(),
            fields,
        }
    }

    pub fn format(&self) -> String {
        let mut union = Generics::default();
        for generics in
            self.fields
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
        for field in &self.fields {
            base.push_str(&field.format_line());
        }
        base.push_str("}\n");
        base
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
