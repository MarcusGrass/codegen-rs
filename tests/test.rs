use codegen_rs::structures::gen_enum::NamedComponentSignature;
use codegen_rs::structures::gen_impl::ImplEntity;
use codegen_rs::structures::gen_struct::{Field, StructEntity, StructKind};
use codegen_rs::structures::generics::{Bound, Bounds, Generic, Generics};
use codegen_rs::structures::method::{Argument, FunctionEntity, Method};
use codegen_rs::structures::visibility::Visibility;
use codegen_rs::structures::Ownership::Owned;
use codegen_rs::structures::{
    Annotation, Annotations, ComponentSignature, Derives, Ownership, RustType, Signature,
    Synchronicity, TypeDef,
};

#[test]
fn creates_a_basic_struct() {
    let struct_entity = StructEntity::new(
        Annotations::empty(),
        Derives::empty(),
        Visibility::Public,
        "MyStruct".to_owned(),
        StructKind::Fields(vec![
            Field::new(
                Visibility::Public,
                NamedComponentSignature::new(
                    "field_a",
                    ComponentSignature::Signature(Signature::simple(RustType::in_scope(
                        "i32".to_owned(),
                    ))),
                ),
            ),
            Field::new(
                Visibility::PublicCrate,
                NamedComponentSignature::new(
                    "field_b",
                    ComponentSignature::Signature(Signature::simple(RustType::in_scope(
                        "u32".to_owned(),
                    ))),
                ),
            ),
            Field::new(
                Visibility::Private,
                NamedComponentSignature::new(
                    "field_c",
                    ComponentSignature::Signature(Signature::simple(RustType::in_scope(
                        "&str".to_owned(),
                    ))),
                ),
            ),
        ]),
    );
    let expect =
        "pub struct MyStruct {\npub field_a: i32,\npub(crate) field_b: u32,\nfield_c: &str,\n}\n"
            .to_string();
    assert_eq!(expect, struct_entity.format())
}

#[test]
fn creates_a_simple_struct_unbounded_generic() {
    let generic = Generic::unbounded("T");
    let struct_entity = StructEntity::new(
        Annotations::empty(),
        Derives::new(vec![
            RustType::in_scope("Copy".to_owned()),
            RustType::in_scope("Clone".to_owned()),
        ]),
        Visibility::Public,
        "MyStruct".to_owned(),
        StructKind::Fields(vec![Field::new(
            Visibility::Public,
            NamedComponentSignature::new("field_a", ComponentSignature::Generic(generic)),
        )]),
    );
    let expect =
        "#[derive(Copy, Clone)]\npub struct MyStruct<T> {\npub field_a: T,\n}\n".to_string();
    assert_eq!(expect, struct_entity.format())
}

#[test]
fn creates_a_struct_with_multiple_bounded_generics() {
    let generic_a = Generic::bounded(
        "T".to_owned(),
        Bounds::single(Bound::required(RustType::in_scope("Debug"))),
    );
    let generic_b = Generic::bounded(
        "R".to_owned(),
        Bounds::multiple(vec![
            Bound::required(RustType::in_scope("Copy".to_owned())),
            Bound::required(RustType::in_scope("Clone".to_owned())),
            Bound::optional(RustType::in_scope("Sized".to_owned())),
        ]),
    );
    let struct_entity = StructEntity::new(
        Annotations::new(vec![
            Annotation::new("cfg(feature = \"default\")".to_owned()),
            Annotation::new("cfg(feature = \"other\")".to_owned()),
        ]),
        Derives::empty(),
        Visibility::Public,
        "MyStruct".to_owned(),
        StructKind::Fields(vec![
            Field::new(
                Visibility::Public,
                NamedComponentSignature::new("field_a", ComponentSignature::Generic(generic_a)),
            ),
            Field::new(
                Visibility::Public,
                NamedComponentSignature::new("field_b", ComponentSignature::Generic(generic_b)),
            ),
        ]),
    );
    let expect = "#[cfg(feature = \"default\")]\n#[cfg(feature = \"other\")]\npub struct MyStruct<T, R> where T: Debug, R: Copy + Clone + ?Sized {\npub field_a: T,\npub field_b: R,\n}\n".to_string();
    assert_eq!(expect, struct_entity.format())
}

#[test]
fn creates_a_basic_function() {
    let func = FunctionEntity::new(
        Annotations::new(vec![Annotation::new("cfg(feature = \"debug\"")]),
        Synchronicity::Sync,
        Visibility::Public,
        "my_func",
        vec![
            Argument::new(
                Ownership::Owned,
                NamedComponentSignature::new(
                    "arg_a",
                    ComponentSignature::Signature(Signature::simple(RustType::in_scope("i32"))),
                ),
            ),
            Argument::new(
                Ownership::Owned,
                NamedComponentSignature::new(
                    "arg_b",
                    ComponentSignature::Signature(Signature::simple(RustType::in_scope("u32"))),
                ),
            ),
        ],
        "println(\"Hello world\");",
        None,
    );
    assert_eq!("#[cfg(feature = \"debug\"]\npub fn my_func(arg_a: i32, arg_b: u32) {\nprintln(\"Hello world\");\n}\n", &func.format());
    //println!("{:?}", func.format());
}

#[test]
fn create_unioned_generic_function() {
    let generic_a = Generic::bounded(
        "T",
        Bounds::single(Bound::required(RustType::in_scope("Copy"))),
    );
    let generic_b = Generic::bounded(
        "R",
        Bounds::single(Bound::optional(RustType::in_scope("Sized"))),
    );
    let generics = Generics::multiple(vec![generic_a.clone(), generic_b.clone()]);
    let func = FunctionEntity::new(
        Annotations::empty(),
        Synchronicity::Async,
        Visibility::Private,
        "my_func",
        vec![
            Argument::new(
                Ownership::Owned,
                NamedComponentSignature::new("arg_a", ComponentSignature::Generic(generic_a)),
            ),
            Argument::new(
                Ownership::Owned,
                NamedComponentSignature::new("arg_b", ComponentSignature::Generic(generic_b)),
            ),
        ],
        "MyType::new()",
        Some(ComponentSignature::Signature(Signature::generic(
            RustType::in_scope("MyType"),
            generics,
        ))),
    );
    assert_eq!("async fn my_func<T, R>(arg_a: T, arg_b: R) -> MyType<T, R> where T: Copy, R: ?Sized {\nMyType::new()\n}\n", func.format());
}

#[test]
fn implement_self() {
    let my_struct = Signature::generic(
        RustType::in_scope("MyStruct"),
        Generics::multiple(vec![Generic::unbounded("T")]),
    );
    let imp = ImplEntity::new(
        Annotations::new(vec![Annotation::new("cfg(feature = \"debug\"")]),
        my_struct.clone(),
        None,
        vec![],
        vec![],
        vec![
            Method::new(
                Annotations::new(vec![Annotation::new("cfg(feature = \"debug\"")]),
                Visibility::Public,
                Synchronicity::Async,
                Some(Ownership::Ref),
                "do_thing",
                vec![],
                Generics::default(),
                Some("println(\"{}\", self);"),
                None,
            ),
            Method::new(
                Annotations::empty(),
                Visibility::PublicCrate,
                Synchronicity::Sync,
                Some(Ownership::MutRef),
                "set_num",
                vec![Argument::new(
                    Ownership::Owned,
                    NamedComponentSignature::new(
                        "num",
                        ComponentSignature::Signature(Signature::simple(RustType::in_scope("i32"))),
                    ),
                )],
                Generics::default(),
                Some("self.num = num;"),
                None,
            ),
            Method::new(
                Annotations::empty(),
                Visibility::Private,
                Synchronicity::Sync,
                Some(Ownership::OwnedMut),
                "transform",
                vec![
                    Argument::new(
                        Owned,
                        NamedComponentSignature::new(
                            "defined_generic",
                            ComponentSignature::Generic(Generic::unbounded("T")),
                        ),
                    ),
                    Argument::new(
                        Owned,
                        NamedComponentSignature::new(
                            "new_generic",
                            ComponentSignature::Generic(Generic::bounded(
                                "R",
                                Bounds::single(Bound::required(RustType::in_scope("Debug"))),
                            )),
                        ),
                    ),
                ],
                Generics::default(),
                Some("self.def = new_generic;\nself.other_thing = new_generic.do_thing();\nself"),
                Some(my_struct.into()),
            ),
        ],
    );
    assert_eq!("#[cfg(feature = \"debug\"]\nimpl<T> MyStruct<T> {\n#[cfg(feature = \"debug\"]\npub async fn do_thing(&self) {\nprintln(\"{}\", self);\n}\n\npub(crate) fn set_num(&mut self, num: i32) {\nself.num = num;\n}\n\nfn transform<R>(mut self, defined_generic: T, new_generic: R) -> MyStruct<T> where R: Debug {\nself.def = new_generic;\nself.other_thing = new_generic.do_thing();\nself\n}\n\n}\n", &imp.format());
}

#[test]
fn implement_for() {
    let my_struct = Signature::generic(
        RustType::in_scope("MyStruct"),
        Generics::multiple(vec![Generic::unbounded("T")]),
    );
    let other_struct = Signature::generic(
        RustType::in_scope("Iterator"),
        Generics::multiple(vec![Generic::unbounded("T")]),
    );
    let imp = ImplEntity::new(
        Annotations::empty(),
        my_struct.clone(),
        Some(other_struct),
        vec![TypeDef::TraitImpl(NamedComponentSignature::new(
            "Item",
            ComponentSignature::Signature(my_struct),
        ))],
        vec![],
        vec![Method::new(
            Annotations::empty(),
            Visibility::Private,
            Synchronicity::Sync,
            Some(Ownership::MutRef),
            "next",
            vec![],
            Generics::default(),
            Some("todo!();"),
            Some(ComponentSignature::Signature(Signature::generic(
                RustType::in_scope("Option"),
                Generics::multiple(vec![Generic::unbounded("T")]),
            ))),
        )],
    );
    assert_eq!("impl<T> Iterator<T> for MyStruct<T>  {\ntype Item = MyStruct<T>;\nfn next(&mut self) -> Option<T> {\ntodo!();\n}\n\n}\n", imp.format());
}
