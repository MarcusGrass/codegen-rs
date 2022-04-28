use crate::structures::method::Method;
use crate::structures::{Annotations, Signature, TypeDef};
use crate::Visibility;

pub struct ImplEntity {
    annotations: Annotations,
    implementor: Signature,
    implementing: Option<Signature>,
    type_defs: Vec<TypeDef>,
    methods: Vec<Method>,
}

impl ImplEntity {
    pub fn format(&self) -> String {
        let diamond = self.implementor.generics.format_diamond_typed();
        let container_owned = self.implementor.generics.clone();
        let mut base = if let Some(implementing) = &self.implementing {
            let container_owned = self.implementor.generics.union(&implementing.generics);
            let union_diamond = container_owned.format_diamond_typed();
            let impl_diamond = implementing.generics.format_diamond_typed();
            format!(
                "{}impl{union_diamond} {}{impl_diamond} for {}{diamond} {} {{\n",
                self.annotations.format(),
                implementing.rust_type.format(),
                self.implementor.rust_type.format(),
                container_owned.format_where_clause()
            )
        } else {
            format!(
                "{}impl{diamond} {}{diamond} {}{{\n",
                self.annotations.format(),
                self.implementor.rust_type.format(),
                self.implementor.generics.format_where_clause()
            )
        };
        for def in &self.type_defs {
            base.push_str(&def.format())
        }
        for method in &self.methods {
            let mut with_container_owned = method.clone();
            if self.implementing.is_some() {
                // When implementing a trait we don't use a modifier here
                with_container_owned.visibility = Visibility::Private;
            }
            with_container_owned.container_inherited_generics = container_owned.clone();
            base.push_str(&format!("{}\n", with_container_owned.format()))
        }
        base.push_str("}\n");
        base
    }
    pub fn new(
        annotations: Annotations,
        implementor: Signature,
        implementing: Option<Signature>,
        type_defs: Vec<TypeDef>,
        methods: Vec<Method>,
    ) -> Self {
        Self {
            annotations,
            implementor,
            implementing,
            type_defs,
            methods,
        }
    }
}
