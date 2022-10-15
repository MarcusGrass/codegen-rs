use crate::structures::gen_enum::NamedComponentSignature;
use crate::structures::generics::Generics;
use crate::structures::visibility::Visibility;
use crate::structures::{Annotations, ComponentSignature, Ownership, Synchronicity};

pub struct FunctionEntity {
    annotations: Annotations,
    synchronicity: Synchronicity,
    visibility: Visibility,
    name: String,
    args: Vec<Argument>,
    body: String,
    return_type: Option<ComponentSignature>,
}

impl FunctionEntity {
    pub fn format(&self) -> String {
        let mut generics = Generics::default();
        let mut arg_spec = vec![];
        for arg in &self.args {
            arg_spec.push(arg.format());
            generics = generics.union(&match &arg.named_sign.component_signature {
                ComponentSignature::Signature(s) => s.get_generics(),
                ComponentSignature::Generic(g) => Generics::multiple(vec![g.clone()]),
            });
        }
        let formatted_args = arg_spec.join(", ");
        let ret = self
            .return_type
            .as_ref()
            .map(|sig| format!(" -> {}", sig.format()))
            .unwrap_or_default();
        let mut function = format!(
            "{}{}{}fn {}{}({}){} {}{{\n",
            self.annotations.format(),
            self.visibility,
            self.synchronicity.format(),
            self.name,
            generics.format(),
            formatted_args,
            ret,
            generics.format_where_clause(),
        );
        function.push_str(&self.body.to_string());
        function.push_str("\n}\n");
        function
    }

    pub fn new(
        annotations: Annotations,
        synchronicity: Synchronicity,
        visibility: Visibility,
        name: impl Into<String>,
        args: Vec<Argument>,
        body: impl Into<String>,
        return_type: Option<ComponentSignature>,
    ) -> Self {
        Self {
            annotations,
            synchronicity,
            visibility,
            name: name.into(),
            args,
            body: body.into(),
            return_type,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Method {
    annotations: Annotations,
    pub(crate) visibility: Visibility,
    synchronicity: Synchronicity,
    self_ownership: Option<Ownership>,
    name: String,
    args: Vec<Argument>,
    // Generics we don't need to specify in where/diamond type
    pub(crate) container_inherited_generics: Generics,
    body: Option<String>,
    return_type: Option<ComponentSignature>,
}

impl Method {
    pub fn format(&self) -> String {
        let mut generics = Generics::default();
        let mut arg_spec = vec![];
        for arg in &self.args {
            arg_spec.push(arg.format());
            let mut pruned_generics = vec![];
            // Extremely inefficient but who cares, this should never be hot
            match &arg.named_sign.component_signature {
                ComponentSignature::Signature(s) => {
                    for arg_generic in &s.get_generics().get_generics() {
                        let mut unique = true;
                        for generic in &self.container_inherited_generics.get_generics() {
                            if generic.alias == arg_generic.alias {
                                unique = false;
                                break;
                            }
                        }
                        if unique {
                            pruned_generics.push(arg_generic.clone());
                        }
                    }
                }
                ComponentSignature::Generic(g) => {
                    let mut unique = true;
                    for generic in &self.container_inherited_generics.get_generics() {
                        if generic.alias == g.alias {
                            unique = false;
                            break;
                        }
                    }
                    if unique {
                        pruned_generics.push(g.clone());
                    }
                }
            }
            generics = generics.union(&Generics::multiple(pruned_generics));
        }
        let self_ownership = self
            .self_ownership
            .map(|so| format!("{}self", so.format()))
            .unwrap_or_default();
        let formatted_args = if arg_spec.is_empty() {
            String::new()
        } else if !self_ownership.is_empty() {
            format!(", {}", arg_spec.join(", "))
        } else {
            arg_spec.join(", ")
        };
        let ret = self
            .return_type
            .as_ref()
            .map(|sig| format!(" -> {}", sig.format()))
            .unwrap_or_default();

        let mut method_base = format!(
            "{}{}{}fn {}{}({}{}){} {}",
            self.annotations.format(),
            self.visibility,
            self.synchronicity.format(),
            self.name,
            generics.format(),
            self_ownership,
            formatted_args,
            ret,
            generics.format_where_clause(),
        );
        if let Some(body) = self.body.as_ref() {
            method_base.push_str("{\n");
            method_base.push_str(&body);
            method_base.push_str("\n}\n");
        } else {
            method_base.push_str(";\n");
        }

        method_base
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        annotations: Annotations,
        visibility: Visibility,
        synchronicity: Synchronicity,
        self_ownership: Option<Ownership>,
        name: impl Into<String>,
        args: Vec<Argument>,
        container_inherited_generics: Generics,
        body: Option<impl Into<String>>,
        return_type: Option<ComponentSignature>,
    ) -> Self {
        Self {
            annotations,
            visibility,
            synchronicity,
            self_ownership,
            name: name.into(),
            args,
            container_inherited_generics,
            body: body.map(|i| i.into()),
            return_type,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Argument {
    ownership: Ownership,
    named_sign: NamedComponentSignature,
}

impl Argument {
    pub fn format(&self) -> String {
        match self.ownership {
            Ownership::Owned | Ownership::Ref | Ownership::MutRef => format!(
                "{}: {}{}",
                self.named_sign.name,
                self.ownership.format(),
                self.named_sign.component_signature.format(),
            ),
            Ownership::OwnedMut => format!(
                "{}{}: {}",
                self.ownership.format(),
                self.named_sign.name,
                self.named_sign.component_signature.format(),
            ),
        }
    }
    pub fn new(ownership: Ownership, named_sign: NamedComponentSignature) -> Self {
        Self {
            ownership,
            named_sign,
        }
    }
}
