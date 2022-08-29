use crate::errors::{Error, Result};

pub fn snake_case(input: &str) -> Result<String> {
    let out = match Case::infer(input)? {
        Case::Pascal => pascal_to_snake(input),
        Case::Kebab => kebab_to_snake(input),
        Case::Camel => camel_to_snake(input),
        Case::Spaced(inner)
        Case::SafeChunk |
        Case::Snake => input.to_string(),
    };
    Ok("".to_owned())
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Case {
    // MyStructsNameIsPascalCased
    Pascal,
    // my_fields_name_is_snake_cased
    Snake,
    // my-directory-name-is-kebab-cased
    Kebab,
    // myJavaVarIsCamelCased
    Camel,
    // word, that will most likely be no-op converted
    SafeChunk,
    Spaced(Box<Case>),
}

impl Case {
    pub fn convert(input: &str, to: Self) -> Result<String> {
        let case = Self::infer(input)?;
    }
    pub fn do_conversion(self, input: &str, to: Self) -> Result<String> {
        if matches!(self, Case::Spaced(_)) {
            panic!("Programming error, infinite recursion!");
        }
        match self {
            Case::Snake => {
                match &to {
                    Case::Pascal => pascal_to_snake(input),
                    Case::Kebab => kebab_to_snake(input),
                    Case::Camel => camel_to_pascal(input),
                    Case::Spaced(s) => s.do_conversion(input, to)?,
                    Case::SafeChunk |
                    Case::Snake => input.to_owned(),
                }
            }
            Case::Pascal => {
                match &to {
                    Case::Snake => snake_to_pascal(input),
                    Case::Kebab => kebab_to_pascal(input),
                    Case::Camel => camel_to_pascal(input),
                    Case::Spaced(s) => s.do_conversion(input, to)?,
                    Case::SafeChunk |
                    Case::Pascal => input.to_string()
                }
            }
            Case::Kebab => {
                match &to {
                    Case::Pascal => kebab_to_pascal(input),
                    Case::Snake => kebab_to_snake(input),
                    Case::Camel => ,
                    Case::Spaced(s) => s.do_conversion(input, to)?,
                    Case::Kebab |
                    Case::SafeChunk => input.to_string(),
                }
            }
            Case::Camel => {}
            Case::SafeChunk => {}
            Case::Spaced(_) => {}
        }
        Ok("".to_owned())
    }
    pub fn infer(input: &str) -> Result<Self> {
        let input = input.trim();
        let mut spaced = false;
        let mut starts_uppercase = false;
        let mut num_uppercases = 0;
        let mut has_dash = false;
        let mut has_underscore = false;
        for (ind, char) in input.chars().enumerate() {
            // No spaces in names
            if char.is_uppercase() && ind == 0 {
                starts_uppercase = true;
                num_uppercases += 1;
            }
            match char {
                ' ' => spaced = true,
                '-' => has_dash = true,
                '_' => has_underscore = true,
                _ => {}
            }
        }
        if num_uppercases == 0 {
            if has_dash && !has_underscore {
                if spaced {
                    Ok(Case::Spaced(Box::new(Case::Kebab)))
                } else {
                    Ok(Case::Kebab)
                }
            } else if has_underscore && !has_dash {
                if spaced {
                    Ok(Case::Spaced(Box::new(Case::Snake)))
                } else {
                    Ok(Case::Snake)
                }
            } else if has_underscore && has_dash {
                Err(fmt_derive_err(
                    input,
                    num_uppercases,
                    starts_uppercase,
                    has_underscore,
                    has_dash,
                    spaced,
                ))
            } else if spaced {
                Ok(Case::Spaced(Box::new(Case::SafeChunk)))
            } else {
                Ok(Case::SafeChunk)
            }
        } else if starts_uppercase {
            if !has_dash && !has_underscore {
                if spaced {
                    Ok(Case::Spaced(Box::new(Case::Pascal)))
                } else {
                    Ok(Case::Pascal)
                }
            } else {
                Err(fmt_derive_err(
                    input,
                    num_uppercases,
                    starts_uppercase,
                    has_underscore,
                    has_dash,
                    spaced,
                ))
            }
        } else {
            if !has_dash && !has_underscore {
                if spaced {
                    Ok(Case::Spaced(Box::new(Case::Camel)))
                } else {
                    Ok(Case::Camel)
                }
            } else {
                Err(fmt_derive_err(
                    input,
                    num_uppercases,
                    starts_uppercase,
                    has_underscore,
                    has_dash,
                    spaced,
                ))
            }
        }
    }
}

#[inline]
fn fmt_derive_err(
    input: &str,
    num_uppercases: usize,
    starts_upper: bool,
    has_underscore: bool,
    has_dash: bool,
    has_space: bool,
) -> Error {
    Error::CaseDerive(input.to_string(), format!("Failed to derive case: uppercase chars = {num_uppercases}, starts uppercase = {starts_upper}, has underscore = {has_underscore}, has dash = {has_dash}, has space = {has_space}"))
}

#[inline]
fn pascal_to_snake(input: &str) -> String {
    let mut transformed = String::with_capacity(input.len());
    for (ind, char) in input.chars().enumerate() {
        if ind == 0 {
            transformed.push_str(&char.to_lowercase().to_string());
        } else if char.is_uppercase() {
            transformed.push('_');
            transformed.push_str(&char.to_lowercase().to_string());
        } else {
            transformed.push(char);
        }
    }
    transformed
}

fn kebab_to_snake(input: &str) -> String {
    input.replace('-', "_")
}

fn camel_to_snake(input: &str) -> String {
    pascal_to_snake(&camel_to_pascal(input))j
}

fn spaced_to_snake(input: &str) -> String {
    input.replace(' ', "_")
}

// ehhhh
#[inline]
fn kebab_to_pascal(input: &str) -> String {
    snake_to_pascal(&kebab_to_snake(input))
}

fn snake_to_pascal(input: &str) -> String {
    let mut transformed = String::with_capacity(input.len());
    let mut last_was_underscore = false;
    for (ind, char) in input.chars().enumerate() {
        if (ind == 0 || last_was_underscore) && char != '_' {
            transformed.push_str(&char.to_uppercase().to_string());
            last_was_underscore = false;
        } else if char == '_' {
            last_was_underscore = true;
        } else {
            transformed.push(char);
        }
    }
    transformed
}

fn camel_to_pascal(input: &str) -> String {
    let mut transformed = String::with_capacity(input.len());
    for (ind, char) in input.chars().enumerate() {
        if ind == 0 {
            transformed.push_str(&char.to_lowercase().to_string());
        } else {
            transformed.push(char);
        }
    }
    transformed
}

#[inline]
fn space_to_pascal(input: &str) -> String {
    snake_to_pascal(&spaced_to_snake(&input))
}

#[inline]
fn pascal_to_kebab(input: &str) -> String {
    snake_to_kebab(&pascal_to_snake(input))
}

#[inline]
fn camel_to_kebab(input: &str) -> String {
    camel_to_snake(&snake_to_kebab(input))
}

fn snake_to_kebab(input: &str) -> String {
    input.replace('_', "-")
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infer() -> Result<()> {
        assert_eq!(Case::SafeChunk, Case::infer("abc")?);
        assert_eq!(Case::SafeChunk, Case::infer(" abc ")?);
        assert_eq!(
            Case::Spaced(Box::new(Case::SafeChunk)),
            Case::infer("one two")?
        );
        assert_eq!(Case::Pascal, Case::infer("Abc")?);
        assert_eq!(Case::Pascal, Case::infer("AbcDef")?);
        Ok(())
    }
}
