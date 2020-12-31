use case::CaseExt;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{alpha1, alphanumeric1, one_of};
use nom::combinator::{eof, map, recognize, value};
use nom::error::{context, ContextError};
use nom::error::{ParseError, VerboseError};
use nom::multi::{fold_many0, many0, many1};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::IResult;

use crate::ast::{Colfer, Field, FieldType, Struct};

fn line_comment<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    preceded(tag("//"), is_not("\n\r"))(input)
}

fn sp<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, (), E> {
    fold_many0(
        alt((value((), one_of(" \t\n\r")), value((), line_comment))),
        (),
        |_, _| (),
    )(input)
}

fn ident<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context(
        "ident",
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        )),
    )(input)
}

fn package<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, String, E> {
    context(
        "package",
        preceded(
            pair(tag("package"), sp),
            terminated(map(ident, |ident| ident.to_snake()), sp),
        ),
    )(input)
}

fn array_type<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, FieldType, E> {
    let f32_ = map(tag("float32"), |_| FieldType::ArrayF32);
    let f64_ = map(tag("float64"), |_| FieldType::ArrayF64);
    let text_ = map(tag("text"), |_| FieldType::ArrayText);
    let binary_ = map(tag("binary"), |_| FieldType::ArrayBinary);
    let s = map(ident, |name| FieldType::ArrayStruct(name.to_camel()));

    context("array-type", alt((f32_, f64_, text_, binary_, s)))(input)
}

fn field_type<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, FieldType, E> {
    let bool_ = map(tag("bool"), |_| FieldType::Bool);
    let u8_ = map(tag("uint8"), |_| FieldType::U8);
    let u16_ = map(tag("uint16"), |_| FieldType::U16);
    let u32_ = map(tag("uint32"), |_| FieldType::U32);
    let u64_ = map(tag("uint64"), |_| FieldType::U64);
    let i32_ = map(tag("int32"), |_| FieldType::I32);
    let i64_ = map(tag("int64"), |_| FieldType::I64);
    let f32_ = map(tag("float32"), |_| FieldType::F32);
    let f64_ = map(tag("float64"), |_| FieldType::F64);
    let timestamp_ = map(tag("timestamp"), |_| FieldType::Timestamp);
    let text_ = map(tag("text"), |_| FieldType::Text);
    let binary_ = map(tag("binary"), |_| FieldType::Binary);

    let array_ = preceded(tuple((tag("["), sp, tag("]"), sp)), array_type);
    let s = map(ident, |name| FieldType::Struct(name.to_camel()));

    context(
        "field-type",
        alt((
            bool_, u8_, u16_, u32_, u64_, i32_, i64_, f32_, f64_, timestamp_, text_, binary_,
            array_, s,
        )),
    )(input)
}

fn type_struct<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, String, E> {
    terminated(
        preceded(pair(tag("type"), sp), map(ident, |ident| ident.to_camel())),
        pair(sp, tag("struct")),
    )(input)
}

fn field_def<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Field, E> {
    delimited(
        sp,
        map(tuple((ident, sp, field_type)), |(name, _, ty)| Field {
            name: {
                let mut name = name.to_snake();
                match name.as_str() {
                    // 2015 strict keywords.
                    | "as" | "break" | "const" | "continue" | "else" | "enum" | "false"
                    | "fn" | "for" | "if" | "impl" | "in" | "let" | "loop" | "match" | "mod" | "move" | "mut"
                    | "pub" | "ref" | "return" | "static" | "struct" | "trait" | "true"
                    | "type" | "unsafe" | "use" | "where" | "while"
                    // 2018 strict keywords.
                    | "dyn"
                    // 2015 reserved keywords.
                    | "abstract" | "become" | "box" | "do" | "final" | "macro" | "override" | "priv" | "typeof"
                    | "unsized" | "virtual" | "yield"
                    // 2018 reserved keywords.
                    | "async" | "await" | "try" => name.insert_str(0, "r#"),
                    // the following keywords are not supported as raw identifiers and are therefore suffixed with an underscore.
                    "self" | "super" | "extern" | "crate" => name += "_",
                    _ => (),
                }
                name
            },
            ty,
        }),
        sp,
    )(input)
}

fn struct_def<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Struct, E> {
    let fields = many1(delimited(sp, field_def, sp));
    let body = delimited(tag("{"), fields, tag("}"));

    map(tuple((type_struct, sp, body)), |(name, _, fields)| Struct {
        name: name.to_string(),
        fields,
    })(input)
}

fn colfer<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Colfer, E> {
    let package = delimited(sp, package, sp);
    let structs = many1(delimited(sp, struct_def, sp));
    context(
        "colfer",
        map(tuple((package, structs, eof)), |(package, structs, _)| {
            Colfer {
                package: package.to_string(),
                structs,
            }
        }),
    )(input)
}

pub fn parse(input: &str) -> Result<Colfer, nom::Err<VerboseError<&str>>> {
    colfer::<VerboseError<&str>>(input).map(|(_, colfer)| colfer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::error::VerboseError;

    #[test]
    fn test_package() {
        assert_eq!(
            package::<VerboseError<&str>>("package MyPkg"),
            Ok(("", "my_pkg".to_string()))
        );
        assert_eq!(
            package::<VerboseError<&str>>("package     MyPkg"),
            Ok(("", "my_pkg".to_string()))
        );
    }

    #[test]
    fn test_field_type() {
        assert_eq!(
            field_type::<VerboseError<&str>>("bool"),
            Ok(("", FieldType::Bool))
        );
        assert_eq!(
            field_type::<VerboseError<&str>>("uint8"),
            Ok(("", FieldType::U8))
        );
        assert_eq!(
            field_type::<VerboseError<&str>>("uint16"),
            Ok(("", FieldType::U16))
        );
        assert_eq!(
            field_type::<VerboseError<&str>>("uint32"),
            Ok(("", FieldType::U32))
        );
        assert_eq!(
            field_type::<VerboseError<&str>>("uint64"),
            Ok(("", FieldType::U64))
        );
        assert_eq!(
            field_type::<VerboseError<&str>>("int32"),
            Ok(("", FieldType::I32))
        );
        assert_eq!(
            field_type::<VerboseError<&str>>("int64"),
            Ok(("", FieldType::I64))
        );
        assert_eq!(
            field_type::<VerboseError<&str>>("float32"),
            Ok(("", FieldType::F32))
        );
        assert_eq!(
            field_type::<VerboseError<&str>>("float64"),
            Ok(("", FieldType::F64))
        );
        assert_eq!(
            field_type::<VerboseError<&str>>("timestamp"),
            Ok(("", FieldType::Timestamp))
        );
        assert_eq!(
            field_type::<VerboseError<&str>>("text"),
            Ok(("", FieldType::Text))
        );
        assert_eq!(
            field_type::<VerboseError<&str>>("binary"),
            Ok(("", FieldType::Binary))
        );
        assert_eq!(
            field_type::<VerboseError<&str>>("abc"),
            Ok(("", FieldType::Struct("Abc".to_string())))
        );

        assert_eq!(
            field_type::<VerboseError<&str>>("[]float32"),
            Ok(("", FieldType::ArrayF32))
        );
        assert_eq!(
            field_type::<VerboseError<&str>>("[ ] float32"),
            Ok(("", FieldType::ArrayF32))
        );
        assert_eq!(
            field_type::<VerboseError<&str>>("[] float32"),
            Ok(("", FieldType::ArrayF32))
        );
    }

    #[test]
    fn test_type_struct() {
        assert_eq!(
            type_struct::<VerboseError<&str>>("type Abc struct"),
            Ok(("", "Abc".to_string()))
        );
        assert_eq!(
            type_struct::<VerboseError<&str>>("type    Abc    struct"),
            Ok(("", "Abc".to_string()))
        );
    }

    #[test]
    fn test_field_def() {
        assert_eq!(
            field_def::<VerboseError<&str>>("abc int32"),
            Ok((
                "",
                Field {
                    name: "abc".to_string(),
                    ty: FieldType::I32
                }
            ))
        );

        assert_eq!(
            field_def::<VerboseError<&str>>("abc       int32"),
            Ok((
                "",
                Field {
                    name: "abc".to_string(),
                    ty: FieldType::I32
                }
            ))
        );
    }

    #[test]
    fn test_struct_def() {
        assert_eq!(
            struct_def::<VerboseError<&str>>(
                r#"type Abc struct {
                value1 int32
                value2 bool
            }"#
            ),
            Ok((
                "",
                Struct {
                    name: "Abc".to_string(),
                    fields: vec![
                        Field {
                            name: "value1".to_string(),
                            ty: FieldType::I32
                        },
                        Field {
                            name: "value2".to_string(),
                            ty: FieldType::Bool
                        }
                    ]
                }
            ))
        )
    }

    #[test]
    fn test_comment() {
        assert_eq!(line_comment::<VerboseError<&str>>("//abc"), Ok(("", "abc")));
    }

    #[test]
    fn test_sp() {
        assert_eq!(sp::<VerboseError<&str>>("//abc"), Ok(("", ())));
    }
}
