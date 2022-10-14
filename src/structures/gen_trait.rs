use crate::{Annotations, Method, Signature, Visibility};
use std::fmt::Write;

#[derive(Debug)]
pub struct TraitEntity {
    annotations: Annotations,
    trait_type: Signature,
    visibility: Visibility,
    methods: Vec<Method>,
    types: Vec<String>,
}

impl TraitEntity {
    pub fn format(&self) -> String {
        let diamond = self.trait_type.generics.format_diamond_typed();
        let container_owned = self.trait_type.generics.clone();
        let mut base = format!(
            "{}{}trait {}{diamond} {}{{\n",
            self.visibility.format(),
            self.annotations.format(),
            self.trait_type.rust_type.format(),
            self.trait_type.generics.format_where_clause()
        );
        for ty in &self.types {
            let _ = base.write_fmt(format_args!("type {};\n", ty));
        }
        for method in &self.methods {
            let mut with_container_owned = method.clone();
            with_container_owned.visibility = Visibility::Private;
            with_container_owned.container_inherited_generics = container_owned.clone();
            let _ = base.write_fmt(format_args!("{}\n", with_container_owned.format()));
        }
        base.push_str("}\n");
        base
    }

    pub fn new(
        annotations: Annotations,
        trait_type: Signature,
        visibility: Visibility,
        methods: Vec<Method>,
        types: Vec<String>,
    ) -> Self {
        Self {
            annotations,
            trait_type,
            visibility,
            methods,
            types,
        }
    }
}
