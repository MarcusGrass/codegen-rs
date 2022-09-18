use crate::errors::{Error, Result};

/// A representation of a case that this lib can infer
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InferCase {
    // MyStructsNameIsPascalCased
    Pascal,
    // my_fields_name_is_snake_cased
    Snake,
    // my-directory-name-is-kebab-cased
    Kebab,
    // myJavaVarIsCamelCased
    Camel,
    // ALSO_KNOWN_AS_SCREAM_CASE
    Scream,
    // WHAT-IS-COBOL-CASE-USED-FOR?
    Cobol,
    // some spaces
    Spaced,
    // WORD, that will most likely be no-op converted after underscoring
    AllCaps,
    // word, that will most likely be no-op converted
    SafeChunk,
}

macro_rules! bail_derive_err {
    ($input: expr, $all_upper: expr, $upper: expr, $starts_upper: expr, $underscore: expr, $dash: expr, $spaced: expr) => {
        return Err(fmt_derive_err(
            $input,
            $all_upper,
            $upper,
            $starts_upper,
            $underscore,
            $dash,
            $spaced,
        ));
    };
}

impl InferCase {
    pub fn infer(input: &str) -> Result<Self> {
        let mut spaced = false;
        let mut starts_uppercase = false;
        let mut num_uppercases = 0;
        let mut all_chars_uppercased = true;
        let mut num_dashes = 0;
        let mut num_underscores = 0;
        for (ind, char) in input.chars().enumerate() {
            // No spaces in names
            if ind == 0 && !char.is_alphabetic() {
                return Err(Error::CaseDerive(
                    input.to_string(),
                    format!("Starts with illegal character {char}"),
                ));
            } else if ind == input.len() - 1 && !char.is_alphanumeric() {
                return Err(Error::CaseDerive(
                    input.to_string(),
                    format!("Ends with illegal character {char}"),
                ));
            } else if char.is_alphanumeric() && !char.is_numeric() && !char.is_uppercase() {
                all_chars_uppercased = false;
            }
            if char.is_numeric() && ind == 0 {}
            if !char.is_numeric() && char.is_uppercase() {
                num_uppercases += 1;
                if ind == 0 {
                    starts_uppercase = true;
                }
            }
            match char {
                ' ' => spaced = true,
                '-' => num_dashes += 1,
                '_' => num_underscores += 1,
                _ => {}
            }
        }
        if all_chars_uppercased {
            if num_dashes == 0 && num_underscores == 0 && !spaced {
                Ok(InferCase::AllCaps)
            } else if num_underscores == 0 && !spaced {
                Ok(InferCase::Cobol)
            } else if num_dashes == 0 && !spaced {
                Ok(InferCase::Scream)
            } else {
                bail_derive_err!(
                    input,
                    all_chars_uppercased,
                    num_uppercases,
                    starts_uppercase,
                    num_underscores,
                    num_dashes,
                    spaced
                );
            }
        } else if num_uppercases == 0 {
            if num_dashes > 0 && num_underscores == 0 {
                if spaced {
                    bail_derive_err!(
                        input,
                        all_chars_uppercased,
                        num_uppercases,
                        starts_uppercase,
                        num_underscores,
                        num_dashes,
                        spaced
                    );
                } else {
                    Ok(InferCase::Kebab)
                }
            } else if num_underscores != 0 && num_dashes == 0 {
                if spaced {
                    bail_derive_err!(
                        input,
                        all_chars_uppercased,
                        num_uppercases,
                        starts_uppercase,
                        num_underscores,
                        num_dashes,
                        spaced
                    );
                } else {
                    Ok(InferCase::Snake)
                }
            } else if num_underscores != 0 && num_dashes != 0 {
                bail_derive_err!(
                    input,
                    all_chars_uppercased,
                    num_uppercases,
                    starts_uppercase,
                    num_underscores,
                    num_dashes,
                    spaced
                );
            } else if spaced {
                Ok(InferCase::Spaced)
            } else {
                Ok(InferCase::SafeChunk)
            }
        } else if starts_uppercase {
            if num_dashes == 0 && num_underscores == 0 {
                if spaced {
                    bail_derive_err!(
                        input,
                        all_chars_uppercased,
                        num_uppercases,
                        starts_uppercase,
                        num_underscores,
                        num_dashes,
                        spaced
                    );
                } else {
                    Ok(InferCase::Pascal)
                }
            } else {
                bail_derive_err!(
                    input,
                    all_chars_uppercased,
                    num_uppercases,
                    starts_uppercase,
                    num_underscores,
                    num_dashes,
                    spaced
                );
            }
        } else if num_dashes == 0 && num_underscores == 0 {
            if spaced {
                bail_derive_err!(
                    input,
                    all_chars_uppercased,
                    num_uppercases,
                    starts_uppercase,
                    num_underscores,
                    num_dashes,
                    spaced
                );
            } else {
                Ok(InferCase::Camel)
            }
        } else {
            bail_derive_err!(
                input,
                all_chars_uppercased,
                num_uppercases,
                starts_uppercase,
                num_underscores,
                num_dashes,
                spaced
            );
        }
    }

    pub fn do_conversion(self, input: &str, to: RustCase) -> Result<String> {
        let out = match self {
            InferCase::Snake => match to {
                RustCase::Pascal => snake_to_pascal(input),
                RustCase::Kebab => snake_to_kebab(input),
                RustCase::Camel => snake_to_camel(input),
                RustCase::Snake => input.to_owned(),
                RustCase::Cobol => snake_to_cobol(input),
                RustCase::Scream => snake_to_scream(input),
            },
            InferCase::Pascal => match to {
                RustCase::Snake => pascal_to_snake(input),
                RustCase::Kebab => pascal_to_kebab(input),
                RustCase::Camel => de_capitalize(input),
                RustCase::Pascal => input.to_string(),
                RustCase::Cobol => pascal_to_cobol(input),
                RustCase::Scream => pascal_to_scream(input),
            },
            InferCase::Kebab => match &to {
                RustCase::Pascal => kebab_to_pascal(input),
                RustCase::Snake => kebab_to_snake(input),
                RustCase::Camel => kebab_to_camel(input),
                RustCase::Kebab => input.to_string(),
                RustCase::Cobol => kebab_to_cobol(input),
                RustCase::Scream => kebab_to_scream(input),
            },
            InferCase::Camel => match &to {
                RustCase::Pascal => capitalize(input),
                RustCase::Snake => camel_to_snake(input),
                RustCase::Kebab => camel_to_kebab(input),
                RustCase::Camel => input.to_string(),
                RustCase::Cobol => camel_to_cobol(input),
                RustCase::Scream => camel_to_scream(input),
            },
            InferCase::SafeChunk => match &to {
                RustCase::Pascal => capitalize(input),
                RustCase::Snake => input.to_string(),
                RustCase::Camel => snake_to_camel(input),
                RustCase::Kebab => input.to_string(),
                RustCase::Cobol => input.to_uppercase(),
                RustCase::Scream => input.to_uppercase(),
            },
            InferCase::Spaced => match to {
                RustCase::Pascal => spaced_to_pascal(input),
                RustCase::Snake => spaced_to_snake(input),
                RustCase::Kebab => spaced_to_kebab(input),
                RustCase::Camel => spaced_to_camel(input),
                RustCase::Cobol => spaced_to_cobol(input),
                RustCase::Scream => spaced_to_scream(input),
            },
            InferCase::AllCaps => {
                let lc = &input.to_lowercase();
                let inf = InferCase::infer(lc)?;
                inf.do_conversion(lc, to)?
            }
            InferCase::Scream => match to {
                RustCase::Pascal => scream_to_pascal(input),
                RustCase::Snake => scream_to_snake(input),
                RustCase::Kebab => scream_to_kebab(input),
                RustCase::Camel => scream_to_camel(input),
                RustCase::Scream => input.to_string(),
                RustCase::Cobol => scream_to_cobol(input),
            },
            InferCase::Cobol => match to {
                RustCase::Pascal => cobol_to_pascal(input),
                RustCase::Snake => cobol_to_snake(input),
                RustCase::Kebab => cobol_to_kebab(input),
                RustCase::Camel => cobol_to_camel(input),
                RustCase::Scream => cobol_to_macro(input),
                RustCase::Cobol => input.to_string(),
            },
        };
        Ok(out)
    }
}

/// A representation of a case that can be output
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RustCase {
    // MyStructsNameIsPascalCased
    Pascal,
    // my_fields_name_is_snake_cased
    Snake,
    // my-directory-name-is-kebab-cased
    Kebab,
    // myJavaVarIsCamelCased
    Camel,
    // ALSO_KNOWN_AS_SCREAM_CASE
    Scream,
    // WHAT-IS-COBOL-CASE-USED-FOR?
    Cobol,
}

impl RustCase {
    pub fn convert(input: &str, to: Self) -> Result<String> {
        let input = input.trim();
        let in_case = InferCase::infer(input)?;
        in_case.do_conversion(input, to)
    }

    pub fn convert_to_valid_rust(input: &str, to: Self) -> Result<String> {
        let s = Self::convert(input, to)?;
        Ok(fix_keyword(&s))
    }
}

#[inline]
fn fmt_derive_err(
    input: &str,
    all_upper: bool,
    num_uppercases: usize,
    starts_upper: bool,
    num_underscores: usize,
    num_dashes: usize,
    has_space: bool,
) -> Error {
    Error::CaseDerive(input.to_string(), format!("Failed to derive case: all_chars_uppercase = {all_upper} uppercase chars = {num_uppercases}, starts uppercase = {starts_upper}, has underscore = {num_underscores}, has dash = {num_dashes}, has space = {has_space}"))
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

#[inline]
fn kebab_to_snake(input: &str) -> String {
    input.replace('-', "_")
}

#[inline]
fn camel_to_snake(input: &str) -> String {
    pascal_to_snake(&capitalize(input))
}

#[inline]
fn spaced_to_snake(input: &str) -> String {
    input.replace(' ', "_")
}

#[inline]
fn scream_to_snake(input: &str) -> String {
    input.to_lowercase()
}

#[inline]
fn cobol_to_snake(input: &str) -> String {
    input.to_lowercase().replace('-', "_")
}

// ehhhh
#[inline]
fn kebab_to_pascal(input: &str) -> String {
    snake_to_pascal(&kebab_to_snake(input))
}

#[inline]
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

#[inline]
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
    snake_to_pascal(&spaced_to_snake(input))
}

#[inline]
fn scream_to_pascal(input: &str) -> String {
    snake_to_pascal(&scream_to_snake(input))
}

#[inline]
fn cobol_to_pascal(input: &str) -> String {
    snake_to_pascal(&cobol_to_snake(input))
}

#[inline]
fn pascal_to_kebab(input: &str) -> String {
    snake_to_kebab(&pascal_to_snake(input))
}

#[inline]
fn camel_to_kebab(input: &str) -> String {
    snake_to_kebab(&camel_to_snake(input))
}

#[inline]
fn snake_to_kebab(input: &str) -> String {
    input.replace('_', "-")
}

#[inline]
fn spaced_to_kebab(input: &str) -> String {
    input.replace(' ', "-")
}

#[inline]
fn scream_to_kebab(input: &str) -> String {
    input.to_lowercase().replace('_', "-")
}

#[inline]
fn cobol_to_kebab(input: &str) -> String {
    input.to_lowercase()
}

#[inline]
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

#[inline]
fn scream_to_camel(input: &str) -> String {
    de_capitalize(&scream_to_pascal(input))
}

#[inline]
fn cobol_to_camel(input: &str) -> String {
    de_capitalize(&cobol_to_pascal(input))
}

#[inline]
fn snake_to_scream(input: &str) -> String {
    input.to_uppercase()
}

#[inline]
fn pascal_to_scream(input: &str) -> String {
    snake_to_scream(&pascal_to_snake(input))
}

#[inline]
fn camel_to_scream(input: &str) -> String {
    snake_to_scream(&camel_to_snake(input))
}

#[inline]
fn kebab_to_scream(input: &str) -> String {
    snake_to_scream(&kebab_to_snake(input))
}

#[inline]
fn spaced_to_scream(input: &str) -> String {
    snake_to_scream(&spaced_to_snake(input))
}

#[inline]
fn cobol_to_macro(input: &str) -> String {
    input.replace('-', "_")
}

#[inline]
fn snake_to_cobol(input: &str) -> String {
    input.to_uppercase().replace('_', "-")
}

#[inline]
fn pascal_to_cobol(input: &str) -> String {
    snake_to_cobol(&pascal_to_snake(input))
}

#[inline]
fn camel_to_cobol(input: &str) -> String {
    snake_to_cobol(&camel_to_snake(input))
}

#[inline]
fn kebab_to_cobol(input: &str) -> String {
    snake_to_cobol(&kebab_to_snake(input))
}

#[inline]
fn spaced_to_cobol(input: &str) -> String {
    snake_to_cobol(&spaced_to_snake(input))
}

#[inline]
fn scream_to_cobol(input: &str) -> String {
    input.replace('_', "-")
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

    fn assert_expected_safe_chunk(expect: &str) -> Result<()> {
        assert_eq!(expect.trim(), RustCase::convert(expect, RustCase::Camel)?);
        assert_eq!(expect.trim(), RustCase::convert(expect, RustCase::Kebab)?);
        assert_eq!(expect.trim(), RustCase::convert(expect, RustCase::Snake)?);
        assert_eq!(
            capitalize(expect.trim()),
            RustCase::convert(expect, RustCase::Pascal)?
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
            assert!(RustCase::convert("my - mixed _ str", case).is_err());
        }
        Ok(())
    }

    const TARGET_CASES: [RustCase; 6] = [
        RustCase::Pascal,
        RustCase::Snake,
        RustCase::Kebab,
        RustCase::Camel,
        RustCase::Cobol,
        RustCase::Scream,
    ];

    trait TestCase {
        fn input(self) -> &'static str;
        fn weird_input(self) -> &'static str;
    }

    impl TestCase for RustCase {
        fn input(self) -> &'static str {
            match self {
                RustCase::Pascal => "MyStruct",
                RustCase::Snake => "my_struct",
                RustCase::Kebab => "my-struct",
                RustCase::Camel => "myStruct",
                RustCase::Scream => "MY_STRUCT",
                RustCase::Cobol => "MY-STRUCT",
            }
        }

        fn weird_input(self) -> &'static str {
            match self {
                RustCase::Pascal => "MYWeIrd ",
                RustCase::Snake => "m_y_we_ird ",
                RustCase::Kebab => "m-y-we-ird ",
                RustCase::Camel => "mYWeIrd ",
                RustCase::Scream => "M_Y_WE_IRD ",
                RustCase::Cobol => "M-Y-WE-IRD ",
            }
        }
    }

    #[test]
    fn test_all_targets() -> Result<()> {
        for input_case in TARGET_CASES {
            test_all_target_conversions(input_case.input(), input_case.weird_input())?;
        }
        Ok(())
    }

    fn test_all_target_conversions(input: &str, input_weird: &str) -> Result<()> {
        let input_case = InferCase::infer(input)?;
        for output_case in TARGET_CASES {
            assert_eq!(
                output_case.input(),
                RustCase::convert(input, output_case)?,
                "Failed regular conversion from {input_case:?} to {output_case:?}, input: '{input}'"
            );
            assert_eq!(
                output_case.weird_input().trim(),
                RustCase::convert(input_weird, output_case)?,
                "Failed weird conversion from {input_case:?} to {output_case:?}, input: '{input}'"
            );
        }
        Ok(())
    }

    #[test]
    fn infer_bad() {
        assert!(InferCase::infer("-hello").is_err());
        assert!(InferCase::infer("_hello").is_err());
        assert!(InferCase::infer("-hello-").is_err());
        assert!(InferCase::infer("_hello_").is_err());
        assert!(InferCase::infer("hello-").is_err());
        assert!(InferCase::infer("hello_").is_err());
    }

    #[test]
    fn test_spaced() {
        let input = "my struct";
        let input_weird = "m y we ird ";
        test_all_target_conversions(input, input_weird).unwrap();
    }
}
