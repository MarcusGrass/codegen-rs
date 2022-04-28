use crate::structures::RustType;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Generics {
    pub(crate) generics: Vec<Generic>,
}

impl Generics {
    pub fn multiple(generics: Vec<Generic>) -> Self {
        Self { generics }
    }

    pub fn single(generic: Generic) -> Self {
        Self {
            generics: vec![generic],
        }
    }

    pub fn single_unbounded(alias: impl Into<String>) -> Self {
        Self {
            generics: vec![Generic::unbounded(alias)],
        }
    }

    pub fn format_where_clause(&self) -> String {
        let mut base = String::new();
        if self.generics.is_empty() {
            base
        } else {
            let sign = self
                .generics
                .iter()
                .filter(|gen| !gen.bounds.bounds.is_empty())
                .map(|gen| gen.format_bounded())
                .collect::<Vec<String>>()
                .join(", ");
            // Kinda stupid way of checking for no bounds but whatever
            if !sign.contains(':') {
                base
            } else {
                base.push_str(&sign);
                format!("where {base} ")
            }
        }
    }

    pub fn format_diamond_typed(&self) -> String {
        let mut base = String::new();
        if self.generics.is_empty() {
            base
        } else {
            let sign = self
                .generics
                .iter()
                .map(|gen| gen.format_diamond_typed())
                .collect::<Vec<String>>()
                .join(", ");
            base.push_str(&sign);
            format!("<{base}>")
        }
    }

    /// Creates a union of two generics preserving order of aliases
    /// Also creates a bounds union on overlapping aliases bounds preserving order of bounds
    /// Upgrades conflicting bounds the same alias have two bounds on the same type where one is optional and the other one isn't
    pub fn union(&self, other: &Generics) -> Generics {
        // TODO: Horribly inefficient
        let mut signatures: HashMap<String, (usize, Vec<Vec<Bound>>)> = HashMap::new();
        let mut it = 0;
        for generic in self.generics.iter().chain(other.generics.iter()) {
            match signatures.entry(generic.alias.clone()) {
                Entry::Occupied(mut o) => {
                    let mut_ref = o.get_mut();
                    mut_ref.1.push(generic.bounds.bounds.clone());
                }
                Entry::Vacant(v) => {
                    v.insert((it, vec![generic.bounds.bounds.clone()]));
                    it += 1;
                }
            }
        }
        let mut generics = vec![];
        for (signature, bounds) in signatures {
            let mut b = vec![];
            let mut b_map: HashMap<RustType, (usize, bool)> = HashMap::new();
            for flat in bounds.1.into_iter().flatten() {
                match b_map.entry(flat.rust_type.clone()) {
                    Entry::Occupied(mut o) => {
                        if o.get_mut().1 && !flat.optional {
                            b[o.get_mut().0] = flat;
                        }
                    }
                    Entry::Vacant(v) => {
                        let ind = b.len();
                        v.insert((ind, flat.optional));
                        b.push(flat);
                    }
                }
            }
            let generic = Generic::bounded(signature, Bounds::multiple(b));
            generics.push((bounds.0, generic));
        }
        generics.sort_by(|a, b| a.0.cmp(&b.0));
        Generics::multiple(generics.into_iter().map(|(_, g)| g).collect())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Generic {
    pub(crate) alias: String,
    bounds: Bounds,
}

impl Generic {
    pub fn into_generics(self) -> Generics {
        Generics::multiple(vec![self])
    }
    pub fn format_bounded(&self) -> String {
        let mut generic_base = self.alias.clone();
        if !self.bounds.bounds.is_empty() {
            generic_base.push_str(": ");
            generic_base.push_str(&self.bounds.format());
        }
        generic_base
    }

    pub fn format_diamond_typed(&self) -> String {
        self.alias.clone()
    }

    pub fn bounded(alias: impl Into<String>, bounds: Bounds) -> Self {
        Self {
            alias: alias.into(),
            bounds,
        }
    }

    pub fn unbounded(alias: impl Into<String>) -> Self {
        Self {
            alias: alias.into(),
            bounds: Bounds::default(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct Bounds {
    pub(crate) bounds: Vec<Bound>,
}

impl Bounds {
    pub fn single(bound: Bound) -> Self {
        Self {
            bounds: vec![bound],
        }
    }

    pub fn multiple(bounds: Vec<Bound>) -> Self {
        Self { bounds }
    }

    pub fn format(&self) -> String {
        self.bounds
            .iter()
            .map(|bound| bound.format())
            .collect::<Vec<String>>()
            .join(" + ")
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Bound {
    rust_type: RustType,
    optional: bool,
}

impl Bound {
    pub fn new(rust_type: RustType, optional: bool) -> Self {
        Self {
            rust_type,
            optional,
        }
    }

    pub fn required(rust_type: RustType) -> Self {
        Self {
            rust_type,
            optional: false,
        }
    }

    pub fn optional(rust_type: RustType) -> Self {
        Self {
            rust_type,
            optional: true,
        }
    }

    pub fn format(&self) -> String {
        if self.optional {
            format!("?{}", self.rust_type.format())
        } else {
            self.rust_type.format()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::structures::generics::{Bound, Bounds};
    use crate::structures::{Generic, Generics, RustType};

    #[test]
    fn generic_format() {
        let generic = Generics::default();
        assert_eq!("", &generic.format_diamond_typed());
        let generic = Generics::multiple(vec![Generic::unbounded("T")]);
        assert_eq!("<T>", &generic.format_diamond_typed());
        let generic = Generics::multiple(vec![Generic::bounded(
            "T",
            Bounds::single(Bound::required(RustType::in_scope("Debug"))),
        )]);
        assert_eq!("<T>", &generic.format_diamond_typed());
        assert_eq!("where T: Debug ", &generic.format_where_clause());
        let generic = Generics::multiple(vec![
            Generic::bounded(
                "T",
                Bounds::single(Bound::required(RustType::in_scope("Debug"))),
            ),
            Generic::unbounded("R"),
            Generic::bounded(
                "V",
                Bounds::single(Bound::optional(RustType::in_scope("Sized"))),
            ),
        ]);
        assert_eq!("<T, R, V>", &generic.format_diamond_typed());
        assert_eq!("where T: Debug, V: ?Sized ", &generic.format_where_clause());
    }

    #[test]
    fn unions() {
        let generics = Generics::default();
        let first_generic = Generics::multiple(vec![Generic::unbounded("T")]);
        let generics = generics.union(&first_generic);
        assert_eq!(1, generics.generics.len());
        assert_eq!("<T>", generics.format_diamond_typed());
        let generics = generics.union(&Generics::multiple(vec![Generic::bounded(
            "R",
            Bounds::single(Bound::required(RustType::in_scope("Debug"))),
        )]));
        assert_eq!(2, generics.generics.len());
        assert_eq!("<T, R>", generics.format_diamond_typed());
        assert_eq!("where R: Debug ", generics.format_where_clause());
        let generics = generics.union(&Generics::multiple(vec![Generic::bounded(
            "R",
            Bounds::single(Bound::optional(RustType::in_scope("Sized"))),
        )]));
        assert_eq!(2, generics.generics.len());
        assert_eq!("<T, R>", generics.format_diamond_typed());
        assert_eq!("where R: Debug + ?Sized ", generics.format_where_clause());
        let generics = generics.union(&Generics::multiple(vec![Generic::bounded(
            "R",
            Bounds::single(Bound::required(RustType::in_scope("Sized"))),
        )]));
        assert_eq!(2, generics.generics.len());
        assert_eq!("<T, R>", generics.format_diamond_typed());
        assert_eq!("where R: Debug + Sized ", generics.format_where_clause());
    }
}
