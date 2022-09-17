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
                Case::Cobol => snake_to_cobol(input),
                Case::Scream => snake_to_scream(input),
                Case::SafeChunk | Case::Spaced | Case::AllCaps => {
                    return Err(fmt_illegal_target(input, to))
                }
            },
            Case::Pascal => match to {
                Case::Snake => pascal_to_snake(input),
                Case::Kebab => pascal_to_kebab(input),
                Case::Camel => de_capitalize(input),
                Case::Pascal => input.to_string(),
                Case::Cobol => pascal_to_cobol(input),
                Case::Scream => pascal_to_scream(input),
                Case::SafeChunk | Case::Spaced | Case::AllCaps => {
                    return Err(fmt_illegal_target(input, to))
                }
            },
            Case::Kebab => match &to {
                Case::Pascal => kebab_to_pascal(input),
                Case::Snake => kebab_to_snake(input),
                Case::Camel => kebab_to_camel(input),
                Case::Kebab => input.to_string(),
                Case::Cobol => kebab_to_cobol(input),
                Case::Scream => kebab_to_scream(input),
                Case::SafeChunk | Case::Spaced | Case::AllCaps => {
                    return Err(fmt_illegal_target(input, to))
                }
            },
            Case::Camel => match &to {
                Case::Pascal => capitalize(input),
                Case::Snake => camel_to_snake(input),
                Case::Kebab => camel_to_kebab(input),
                Case::Camel => input.to_string(),
                Case::Cobol => camel_to_cobol(input),
                Case::Scream => camel_to_scream(input),
                Case::SafeChunk | Case::Spaced | Case::AllCaps => {
                    return Err(fmt_illegal_target(input, to))
                }
            },
            Case::SafeChunk => match &to {
                Case::Pascal => capitalize(input),
                Case::Snake => input.to_string(),
                Case::Camel => snake_to_camel(input),
                Case::Kebab => input.to_string(),
                Case::Cobol => input.to_uppercase(),
                Case::Scream => input.to_uppercase(),
                Case::SafeChunk | Case::Spaced | Case::AllCaps => {
                    return Err(fmt_illegal_target(input, to))
                }
            },
            Case::Spaced => match to {
                Case::Pascal => spaced_to_pascal(input),
                Case::Snake => spaced_to_snake(input),
                Case::Kebab => spaced_to_kebab(input),
                Case::Camel => spaced_to_camel(input),
                Case::Cobol => spaced_to_cobol(input),
                Case::Scream => spaced_to_scream(input),
                Case::SafeChunk | Case::Spaced | Case::AllCaps => {
                    return Err(fmt_illegal_target(input, to))
                }
            },
            Case::AllCaps => match to {
                Case::Pascal
                | Case::Snake
                | Case::Kebab
                | Case::Camel
                | Case::Scream
                | Case::Cobol => Case::SafeChunk.do_conversion(&input.to_lowercase(), to)?,
                Case::SafeChunk | Case::Spaced | Case::AllCaps => {
                    return Err(fmt_illegal_target(input, to))
                }
            },
            Case::Scream => match to {
                Case::Pascal => scream_to_pascal(input),
                Case::Snake => scream_to_snake(input),
                Case::Kebab => scream_to_kebab(input),
                Case::Camel => scream_to_camel(input),
                Case::Scream => input.to_string(),
                Case::Cobol => scream_to_cobol(input),
                Case::SafeChunk | Case::AllCaps | Case::Spaced => {
                    return Err(fmt_illegal_target(input, to))
                }
            },
            Case::Cobol => match to {
                Case::Pascal => cobol_to_pascal(input),
                Case::Snake => cobol_to_snake(input),
                Case::Kebab => cobol_to_kebab(input),
                Case::Camel => cobol_to_camel(input),
                Case::Scream => cobol_to_macro(input),
                Case::Cobol => input.to_string(),
                Case::SafeChunk | Case::AllCaps | Case::Spaced => {
                    return Err(fmt_illegal_target(input, to))
                }
            },
        };
        Ok(out)
    }
    pub fn infer(input: &str) -> Result<Self> {
        let mut spaced = false;
        let mut starts_uppercase = false;
        let mut num_uppercases = 0;
        let mut all_chars_uppercased = true;
        let mut num_dashes = 0;
        let mut num_underscores = 0;
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
            } else if char.is_alphanumeric() && !char.is_uppercase() {
                all_chars_uppercased = false;
            }
            if char.is_uppercase() {
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
                Ok(Case::AllCaps)
            } else if num_underscores == 0 && !spaced {
                Ok(Case::Cobol)
            } else if num_dashes == 0 && !spaced {
                Ok(Case::Scream)
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
                    Ok(Case::Kebab)
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
                    Ok(Case::Snake)
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
                Ok(Case::Spaced)
            } else {
                Ok(Case::SafeChunk)
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
                    Ok(Case::Pascal)
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
                Ok(Case::Camel)
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

    const ALL_CASES: [Case; 9] = [
        Case::Pascal,
        Case::Snake,
        Case::Kebab,
        Case::Camel,
        Case::Scream,
        Case::Cobol,
        Case::Spaced,
        Case::AllCaps,
        Case::SafeChunk,
    ];

    const TARGET_CASES: [Case; 6] = [
        Case::Pascal,
        Case::Snake,
        Case::Kebab,
        Case::Camel,
        Case::Cobol,
        Case::Scream,
    ];

    trait TestCase {
        fn input(self) -> &'static str;
        fn expect_regular(self) -> Option<&'static str>;
        fn weird_input(self) -> &'static str;
        fn expect_weird(self) -> Option<&'static str>;
        fn this(self) -> Case;
    }

    impl TestCase for Case {
        fn input(self) -> &'static str {
            match self {
                Case::Pascal => "MyStruct",
                Case::Snake => "my_struct",
                Case::Kebab => "my-struct",
                Case::Camel => "myStruct",
                Case::Scream => "MY_STRUCT",
                Case::Cobol => "MY-STRUCT",
                Case::Spaced => "my struct",
                Case::AllCaps => "MYSTRUCT",
                Case::SafeChunk => "mystruct",
            }
        }

        fn expect_regular(self) -> Option<&'static str> {
            match self {
                Case::Pascal
                | Case::Snake
                | Case::Kebab
                | Case::Camel
                | Case::Scream
                | Case::Cobol => Some(self.input()),
                Case::Spaced | Case::AllCaps | Case::SafeChunk => None,
            }
        }

        fn weird_input(self) -> &'static str {
            match self {
                Case::Pascal => "MYWeIrd ",
                Case::Snake => "m_y_we_ird ",
                Case::Kebab => "m-y-we-ird ",
                Case::Camel => "mYWeIrd ",
                Case::Scream => "M_Y_WE_IRD ",
                Case::Cobol => "M-Y-WE-IRD ",
                Case::Spaced => "m y we ird ",
                Case::AllCaps => "MYWEIRD",
                Case::SafeChunk => "myweird",
            }
        }

        fn expect_weird(self) -> Option<&'static str> {
            match self {
                Case::Pascal
                | Case::Snake
                | Case::Kebab
                | Case::Camel
                | Case::Scream
                | Case::Cobol => Some(self.weird_input()),
                Case::Spaced | Case::AllCaps | Case::SafeChunk => None,
            }
        }

        fn this(self) -> Case {
            self
        }
    }

    #[test]
    fn test_all() -> Result<()> {
        for input_case in ALL_CASES {
            for output_case in TARGET_CASES {
                eprintln!("Try {input_case:?} -> {output_case:?}");
                if let Some(expect_regular) = output_case.expect_regular() {
                    let input = input_case.input();
                    let case = output_case.this();
                    assert_eq!(
                        expect_regular,
                        Case::convert(input, case)?,
                        "Failed regular conversion from {input_case:?} to {case:?}, input: '{input}'"
                    );
                }
                if let Some(expect_weird) = output_case.expect_weird() {
                    let input = input_case.weird_input();
                    let case = output_case.this();
                    assert_eq!(
                        expect_weird.trim(),
                        Case::convert(input, case)?,
                        "Failed weird conversion from {input_case:?} to {case:?}, input: '{input}'"
                    );
                }
                eprintln!("Done {:?} -> {:?}", input_case, output_case);
            }
        }
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
}
