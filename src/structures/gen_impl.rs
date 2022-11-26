use crate::structures::generics::{Generic, Generics};
use crate::structures::method::Method;
use crate::structures::{Annotations, Signature, TypeDef};
use crate::{ConstantEntity, Visibility};
use std::fmt::Write;

pub struct ImplEntity {
    annotations: Annotations,
    implementor: Signature,
    implementing: Option<Signature>,
    type_defs: Vec<TypeDef>,
    consts: Vec<ConstantEntity>,
    methods: Vec<Method>,
}

impl ImplEntity {
    pub fn format(&self) -> String {
        let diamond = self.implementor.get_associated_generics().format();
        let container_owned = self.implementor.get_generics();
        let mut base = if let Some(implementing) = &self.implementing {
            let container_owned = self
                .implementor
                .get_generics()
                .union(&implementing.get_generics());
            let union_diamond = container_owned.format();
            let impl_diamond: Vec<Generic> = implementing
                .get_generics()
                .get_generics()
                .into_iter()
                // Extremely inefficient
                .filter(|g| !container_owned.get_generics().contains(g))
                .collect();
            let impl_diamond = Generics::multiple(impl_diamond).format();
            let diamond: Vec<Generic> = self
                .implementor
                .get_associated_generics()
                .get_generics()
                .into_iter()
                .filter(|g| !container_owned.get_generics().contains(g))
                .collect();
            let diamond = Generics::multiple(diamond).format();
            format!(
                "{}impl{union_diamond} {}{impl_diamond} for {}{diamond} {} {{\n",
                self.annotations.format(),
                implementing.format(),
                self.implementor.format(),
                container_owned.format_where_clause()
            )
        } else {
            format!(
                "{}impl{diamond} {}{diamond} {}{{\n",
                self.annotations.format(),
                self.implementor.get_any_alias(),
                self.implementor.get_generics().format_where_clause()
            )
        };
        for cnst in &self.consts {
            base.push_str(&cnst.format());
        }
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
            let _ = base.write_fmt(format_args!("{}\n", with_container_owned.format()));
        }
        base.push_str("}\n");
        base
    }
    pub fn new(
        annotations: Annotations,
        implementor: Signature,
        implementing: Option<Signature>,
        type_defs: Vec<TypeDef>,
        consts: Vec<ConstantEntity>,
        methods: Vec<Method>,
    ) -> Self {
        Self {
            annotations,
            implementor,
            implementing,
            type_defs,
            consts,
            methods,
        }
    }
}
