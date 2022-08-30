use crate::errors::{Error, Result};

/// A representation of a case
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
    Spaced,
}

macro_rules! bail_derive_err {
    ($input: expr, $upper: expr, $starts_upper: expr, $underscore: expr, $dash: expr, $spaced: expr) => {
        return Err(fmt_derive_err(
            $input,
            $upper,
            $starts_upper,
            $underscore,
            $dash,
            $spaced,
        ));
    };
}

impl Case {
    pub fn convert(input: &str, to: Self) -> Result<String> {
        let input = input.trim();
        let case = Self::infer(input)?;
        case.do_conversion(input, to)
    }

    pub fn convert_to_valid_rust(input: &str, to: Self) -> Result<String> {
        let s = Self::convert(input, to)?;
        Ok(fix_keyword(&s))
    }
    pub fn do_conversion(self, input: &str, to: Self) -> Result<String> {
        let out = match self {
            Case::Snake => match to {
                Case::Pascal => snake_to_pascal(input),
                Case::Kebab => snake_to_kebab(input),
                Case::Camel => snake_to_camel(input),
                Case::Snake => input.to_owned(),
                Case::SafeChunk | Case::Spaced => return Err(fmt_illegal_target(input, to)),
            },
            Case::Pascal => match to {
                Case::Snake => pascal_to_snake(input),
                Case::Kebab => pascal_to_kebab(input),
                Case::Camel => de_capitalize(input),
                Case::Pascal => input.to_string(),
                Case::SafeChunk | Case::Spaced => return Err(fmt_illegal_target(input, to)),
            },
            Case::Kebab => match &to {
                Case::Pascal => kebab_to_pascal(input),
                Case::Snake => kebab_to_snake(input),
                Case::Camel => kebab_to_camel(input),
                Case::Kebab => input.to_string(),
                Case::SafeChunk | Case::Spaced => return Err(fmt_illegal_target(input, to)),
            },
            Case::Camel => match &to {
                Case::Pascal => capitalize(input),
                Case::Snake => camel_to_snake(input),
                Case::Kebab => camel_to_kebab(input),
                Case::Camel => input.to_string(),
                Case::SafeChunk | Case::Spaced => return Err(fmt_illegal_target(input, to)),
            },
            Case::SafeChunk => match &to {
                Case::Pascal => capitalize(input),
                Case::Snake => camel_to_snake(input),
                Case::Camel => camel_to_kebab(input),
                Case::Kebab => input.to_string(),
                Case::SafeChunk | Case::Spaced => return Err(fmt_illegal_target(input, to)),
            },
            Case::Spaced => match to {
                Case::Pascal => spaced_to_pascal(input),
                Case::Snake => spaced_to_snake(input),
                Case::Kebab => spaced_to_kebab(input),
                Case::Camel => spaced_to_camel(input),
                Case::SafeChunk | Case::Spaced => return Err(fmt_illegal_target(input, to)),
            },
        };
        Ok(out)
    }
    pub fn infer(input: &str) -> Result<Self> {
        let mut spaced = false;
        let mut starts_uppercase = false;
        let mut num_uppercases = 0;
        let mut has_dash = false;
        let mut has_underscore = false;
        for (ind, char) in input.chars().enumerate() {
            // No spaces in names
            if ind == 0 && !char.is_alphanumeric() {
                return Err(Error::CaseDerive(
                    input.to_string(),
                    format!("Starts with illegal character {char}"),
                ));
            } else if ind == input.len() - 1 && !char.is_alphanumeric() {
                return Err(Error::CaseDerive(
                    input.to_string(),
                    format!("Ends with illegal character {char}"),
                ));
            }
            if char.is_uppercase() {
                num_uppercases += 1;
                if ind == 0 {
                    starts_uppercase = true;
                }
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
                    bail_derive_err!(
                        input,
                        num_uppercases,
                        starts_uppercase,
                        has_underscore,
                        has_dash,
                        spaced
                    );
                } else {
                    Ok(Case::Kebab)
                }
            } else if has_underscore && !has_dash {
                if spaced {
                    bail_derive_err!(
                        input,
                        num_uppercases,
                        starts_uppercase,
                        has_underscore,
                        has_dash,
                        spaced
                    );
                } else {
                    Ok(Case::Snake)
                }
            } else if has_underscore && has_dash {
                bail_derive_err!(
                    input,
                    num_uppercases,
                    starts_uppercase,
                    has_underscore,
                    has_dash,
                    spaced
                );
            } else if spaced {
                Ok(Case::Spaced)
            } else {
                Ok(Case::SafeChunk)
            }
        } else if starts_uppercase {
            if !has_dash && !has_underscore {
                if spaced {
                    bail_derive_err!(
                        input,
                        num_uppercases,
                        starts_uppercase,
                        has_underscore,
                        has_dash,
                        spaced
                    );
                } else {
                    Ok(Case::Pascal)
                }
            } else {
                bail_derive_err!(
                    input,
                    num_uppercases,
                    starts_uppercase,
                    has_underscore,
                    has_dash,
                    spaced
                );
            }
        } else if !has_dash && !has_underscore {
            if spaced {
                bail_derive_err!(
                    input,
                    num_uppercases,
                    starts_uppercase,
                    has_underscore,
                    has_dash,
                    spaced
                );
            } else {
                Ok(Case::Camel)
            }
        } else {
            bail_derive_err!(
                input,
                num_uppercases,
                starts_uppercase,
                has_underscore,
                has_dash,
                spaced
            );
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
fn fmt_illegal_target(input: &str, target: Case) -> Error {
    Error::CaseConvert(
        input.to_string(),
        format!("{target:?}"),
        "Illegal target".to_owned(),
    )
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
    pascal_to_snake(&capitalize(input))
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

fn capitalize(input: &str) -> String {
    let mut transformed = String::with_capacity(input.len());
    for (ind, char) in input.chars().enumerate() {
        if ind == 0 {
            transformed.push_str(&char.to_uppercase().to_string());
        } else {
            transformed.push(char);
        }
    }
    transformed
}

#[inline]
fn spaced_to_pascal(input: &str) -> String {
    snake_to_pascal(&spaced_to_snake(&input))
}

#[inline]
fn pascal_to_kebab(input: &str) -> String {
    snake_to_kebab(&pascal_to_snake(input))
}

#[inline]
fn camel_to_kebab(input: &str) -> String {
    snake_to_kebab(&camel_to_snake(input))
}

fn snake_to_kebab(input: &str) -> String {
    input.replace('_', "-")
}

fn spaced_to_kebab(input: &str) -> String {
    input.replace(' ', "-")
}

fn de_capitalize(input: &str) -> String {
    let mut transformed = String::new();
    for (ind, char) in input.chars().enumerate() {
        if ind == 0 {
            transformed.push_str(&char.to_lowercase().to_string())
        } else {
            transformed.push(char)
        }
    }
    transformed
}

#[inline]
fn kebab_to_camel(input: &str) -> String {
    de_capitalize(&kebab_to_pascal(input))
}

#[inline]
fn snake_to_camel(input: &str) -> String {
    de_capitalize(&snake_to_pascal(input))
}

#[inline]
fn spaced_to_camel(input: &str) -> String {
    de_capitalize(&spaced_to_pascal(input))
}

const KEYWORDS: [&str; 51] = [
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
];

pub fn fix_keyword(input: &str) -> String {
    // inefficient but whatever i'm not pulling in lazy_static
    if KEYWORDS.contains(&input) {
        format!("r#{input}")
    } else {
        input.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TARGET_CASES: [Case; 4] = [Case::Camel, Case::Kebab, Case::Pascal, Case::Snake];

    fn assert_expected_safe_chunk(expect: &str) -> Result<()> {
        assert_eq!(expect.trim(), Case::convert(expect, Case::Camel)?);
        assert_eq!(expect.trim(), Case::convert(expect, Case::Kebab)?);
        assert_eq!(expect.trim(), Case::convert(expect, Case::Snake)?);
        assert_eq!(
            capitalize(expect.trim()),
            Case::convert(expect, Case::Pascal)?
        );
        Ok(())
    }

    #[test]
    fn convert_basic() -> Result<()> {
        assert_expected_safe_chunk("abc")?;
        assert_expected_safe_chunk(" abc")?;
        assert_expected_safe_chunk(" abc ")?;

        // No mixed stuff
        for case in TARGET_CASES {
            assert!(Case::convert("my - mixed _ str", case).is_err());
        }
        Ok(())
    }

    const MY_STRUCT_SPACED: &str = "my struct";
    const MY_STRUCT_SNAKE: &str = "my_struct";
    const MY_STRUCT_KEBAB: &str = "my-struct";
    const MY_STRUCT_PASCAL: &str = "MyStruct";
    const MY_STRUCT_CAMEL: &str = "myStruct";

    fn assert_conversion_my_struct(my_struct: &str) -> Result<()> {
        assert_eq!(MY_STRUCT_SNAKE, Case::convert(my_struct, Case::Snake)?);
        assert_eq!(MY_STRUCT_KEBAB, Case::convert(my_struct, Case::Kebab)?);
        assert_eq!(MY_STRUCT_CAMEL, Case::convert(my_struct, Case::Camel)?);
        assert_eq!(MY_STRUCT_PASCAL, Case::convert(my_struct, Case::Pascal)?);
        assert!(Case::convert(my_struct, Case::SafeChunk).is_err());
        assert!(Case::convert(my_struct, Case::Spaced).is_err());
        Ok(())
    }

    const MY_WEIRD_SPACED: &str = "m y we ird ";
    const MY_WEIRD_SNAKE: &str = "m_y_we_ird ";
    const MY_WEIRD_KEBAB: &str = "m-y-we-ird ";
    const MY_WEIRD_CAMEL: &str = "mYWeIrd ";
    const MY_WEIRD_PASCAL: &str = "MYWeIrd ";

    fn assert_conversion_my_weird(my_struct: &str) -> Result<()> {
        assert_eq!(
            MY_WEIRD_SNAKE.trim(),
            Case::convert(my_struct, Case::Snake)?
        );
        assert_eq!(
            MY_WEIRD_KEBAB.trim(),
            Case::convert(my_struct, Case::Kebab)?
        );
        assert_eq!(
            MY_WEIRD_CAMEL.trim(),
            Case::convert(my_struct, Case::Camel)?
        );
        assert_eq!(
            MY_WEIRD_PASCAL.trim(),
            Case::convert(my_struct, Case::Pascal)?
        );
        assert!(Case::convert(my_struct, Case::SafeChunk).is_err());
        assert!(Case::convert(my_struct, Case::Spaced).is_err());
        Ok(())
    }

    #[test]
    fn infer_bad() {
        assert!(Case::infer("-hello").is_err());
        assert!(Case::infer("_hello").is_err());
        assert!(Case::infer("-hello-").is_err());
        assert!(Case::infer("_hello_").is_err());
        assert!(Case::infer("hello-").is_err());
        assert!(Case::infer("hello_").is_err());
    }

    #[test]
    fn convert_from_spaced() -> Result<()> {
        assert_conversion_my_struct(MY_STRUCT_SPACED)?;
        assert_conversion_my_weird(MY_WEIRD_SPACED)
    }

    #[test]
    fn convert_from_kebab() -> Result<()> {
        assert_conversion_my_struct(MY_STRUCT_KEBAB)?;
        assert_conversion_my_weird(MY_WEIRD_KEBAB)
    }

    #[test]
    fn convert_from_snake() -> Result<()> {
        assert_conversion_my_struct(MY_STRUCT_SNAKE)?;
        assert_conversion_my_weird(MY_WEIRD_SNAKE)
    }

    #[test]
    fn convert_from_camel() -> Result<()> {
        assert_conversion_my_struct(MY_STRUCT_CAMEL)?;
        assert_conversion_my_weird(MY_WEIRD_CAMEL)
    }

    #[test]
    fn convert_from_pascal() -> Result<()> {
        assert_conversion_my_struct(MY_STRUCT_PASCAL)?;
        assert_conversion_my_weird(MY_WEIRD_PASCAL)
    }
}
