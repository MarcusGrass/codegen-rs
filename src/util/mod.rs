pub mod casing;
macro_rules! add_annotation {
    () => {
        pub fn add_annotation(mut self, annotation: crate::structures::Annotation) -> Self {
            self.annotations.annotations.push(annotation);
            self
        }

        pub fn add_simple_annotation(mut self, annotation: impl Into<String>) -> Self {
            self.annotations
                .annotations
                .push($crate::structures::Annotation::new(annotation));
            self
        }
    };
}

macro_rules! add_derive {
    () => {
        pub fn add_derive_in_scope(mut self, type_name: impl Into<String>) -> Self {
            self.derives.rust_types.push(RustType::in_scope(type_name));
            self
        }
        pub fn add_derive(mut self, rust_type: RustType) -> Self {
            self.derives.rust_types.push(rust_type);
            self
        }
    };
}

macro_rules! set_async {
    () => {
        pub fn set_async(mut self) -> Self {
            self.synchronicity = crate::structures::Synchronicity::Async;
            self
        }
    };
}

macro_rules! set_visibility {
    () => {
        pub fn set_visibility(
            mut self,
            visibility: crate::structures::visibility::Visibility,
        ) -> Self {
            self.visibility = visibility;
            self
        }
    };
}
macro_rules! add_argument {
    () => {
        pub fn add_argument(mut self, arg: crate::structures::method::Argument) -> Self {
            self.args.push(arg);
            self
        }
        pub fn add_argument_in_scope_simple_type(
            mut self,
            ownership: Ownership,
            alias: impl Into<String>,
            type_name: impl Into<String>,
        ) -> Self {
            let ncs = NamedComponentSignature::new(
                alias.into(),
                ComponentSignature::Signature(Signature::simple(RustType::in_scope(
                    type_name.into(),
                ))),
            );
            let arg = Argument::new(ownership, ncs);
            self.args.push(arg);
            self
        }
        pub fn add_argument_unbounded_generic(
            mut self,
            ownership: Ownership,
            arg_alias: impl Into<String>,
            generic_alias: impl Into<String>,
        ) -> Self {
            self.args.push(Argument::new(
                ownership,
                NamedComponentSignature::new(
                    arg_alias.into(),
                    ComponentSignature::Generic(Generic::unbounded(generic_alias)),
                ),
            ));
            self
        }
        pub fn add_argument_bounded_generic(
            mut self,
            ownership: Ownership,
            arg_alias: impl Into<String>,
            generic_alias: impl Into<String>,
            bounds: Bounds,
        ) -> Self {
            self.args.push(Argument::new(
                ownership,
                NamedComponentSignature::new(
                    arg_alias.into(),
                    ComponentSignature::Generic(Generic::bounded(generic_alias, bounds)),
                ),
            ));
            self
        }
    };
}

macro_rules! set_return_type {
    () => {
        pub fn set_return_type(
            mut self,
            return_type: crate::structures::ComponentSignature,
        ) -> Self {
            self.return_type = Some(return_type);
            self
        }
    };
}

macro_rules! set_body {
    () => {
        pub fn set_body(mut self, body: impl Into<String>) -> Self {
            self.body = body.into();
            self
        }
    };
}
