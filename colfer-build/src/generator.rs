use std::fmt::Write;

use crate::ast::{Colfer, FieldType};

pub fn generate(colfer: &Colfer) -> anyhow::Result<String> {
    let mut code = String::new();

    writeln!(
        &mut code,
        "#![allow(unused_variables, unused_assignments, unused_mut, unused_imports)]"
    )?;
    writeln!(&mut code)?;
    writeln!(&mut code, "use std::io::{{Write, Read, Result}};")?;
    writeln!(&mut code)?;
    writeln!(&mut code, "use colfer::{{Message, Type, DateTime}};")?;
    writeln!(&mut code)?;

    for s in &colfer.structs {
        writeln!(&mut code, "#[derive(Default, Clone, Debug, PartialEq)]")?;
        writeln!(&mut code, "pub struct {} {{", s.name)?;

        for f in &s.fields {
            write!(&mut code, "\tpub {}: ", f.name)?;

            match &f.ty {
                FieldType::Bool => write!(&mut code, "bool")?,
                FieldType::U8 => write!(&mut code, "u8")?,
                FieldType::U16 => write!(&mut code, "u16")?,
                FieldType::U32 => write!(&mut code, "u32")?,
                FieldType::U64 => write!(&mut code, "u64")?,
                FieldType::I32 => write!(&mut code, "i32")?,
                FieldType::I64 => write!(&mut code, "i64")?,
                FieldType::F32 => write!(&mut code, "f32")?,
                FieldType::F64 => write!(&mut code, "f64")?,
                FieldType::Timestamp => write!(&mut code, "DateTime")?,
                FieldType::Text => write!(&mut code, "String")?,
                FieldType::Binary => write!(&mut code, "Vec<u8>")?,
                FieldType::Struct(name) => {
                    if colfer.need_box(&s.name, &name) {
                        write!(&mut code, "Option<Box<{}>>", name)?;
                    } else {
                        write!(&mut code, "Option<{}>", name)?;
                    }
                }
                FieldType::ArrayF32 => write!(&mut code, "Vec<f32>")?,
                FieldType::ArrayF64 => write!(&mut code, "Vec<f64>")?,
                FieldType::ArrayText => write!(&mut code, "Vec<String>")?,
                FieldType::ArrayBinary => write!(&mut code, "Vec<Vec<u8>>")?,
                FieldType::ArrayStruct(name) => write!(&mut code, "Vec<{}>", name)?,
            }

            writeln!(&mut code, ",")?;
        }

        writeln!(&mut code, "}}")?;

        writeln!(&mut code)?;
        writeln!(&mut code, "impl Message for {} {{", s.name)?;

        writeln!(
            &mut code,
            "\tfn encode<W: Write>(&self, w: &mut W) -> Result<()> {{"
        )?;
        for (idx, f) in s.fields.iter().enumerate() {
            match &f.ty {
                FieldType::Struct(_) => writeln!(
                    &mut code,
                    "\t\tcolfer::encode_message(w, {}, self.{}.as_deref())?;",
                    idx, f.name
                )?,
                FieldType::ArrayStruct(_) => writeln!(
                    &mut code,
                    "\t\tcolfer::encode_messages(w, {}, &self.{})?;",
                    idx, f.name
                )?,
                _ => writeln!(&mut code, "\t\tself.{}.encode(w, {})?;", f.name, idx)?,
            }
        }
        writeln!(&mut code)?;
        writeln!(&mut code, "\t\tOk(())\n\t}}")?;
        writeln!(&mut code)?;

        writeln!(
            &mut code,
            "\tfn decode<R: Read>(r: &mut R) -> Result<Self> {{"
        )?;
        writeln!(&mut code, "\t\tlet mut obj = Self::default();")?;
        writeln!(
            &mut code,
            "\t\tlet (mut id, mut flag) = colfer::read_header(r)?;"
        )?;
        for (idx, f) in s.fields.iter().enumerate() {
            writeln!(&mut code, "\t\tif id == {} {{", idx)?;
            match &f.ty {
                FieldType::Struct(_) => writeln!(
                    &mut code,
                    "\t\t\tobj.{} = colfer::decode_message(r)?;",
                    f.name,
                )?,
                FieldType::ArrayStruct(_) => writeln!(
                    &mut code,
                    "\t\t\tobj.{} = colfer::decode_messages(r)?;",
                    f.name,
                )?,
                _ => writeln!(&mut code, "\t\t\tobj.{} = Type::decode(r, flag)?;", f.name)?,
            }
            if idx < s.fields.len() - 1 {
                writeln!(&mut code, "\t\t\tlet next = colfer::read_header(r)?;")?;
                writeln!(&mut code, "\t\t\tid = next.0;")?;
                writeln!(&mut code, "\t\t\tflag = next.1;")?;
            }
            writeln!(&mut code, "\t\t}}")?;
        }
        writeln!(&mut code)?;

        writeln!(&mut code, "\t\tOk(obj)\n\t}}")?;

        writeln!(&mut code)?;
        writeln!(&mut code, "\tfn size(&self) -> usize {{")?;
        writeln!(&mut code, "\t\tlet mut size = 0;")?;
        for f in &s.fields {
            match &f.ty {
                FieldType::Struct(_) => {
                    writeln!(
                        &mut code,
                        "\t\tsize += self.{}.as_ref().map(|s| s.size()).unwrap_or_default();",
                        f.name
                    )?;
                }
                FieldType::ArrayStruct(_) => {
                    writeln!(
                        &mut code,
                        "\t\tsize += self.{}.iter().map(|s| s.size()).sum::<usize>();",
                        f.name
                    )?;
                }
                _ => {
                    writeln!(&mut code, "\t\tsize += self.{}.size();", f.name)?;
                }
            }
        }
        writeln!(&mut code, "\t\tsize")?;
        writeln!(&mut code, "\t}}")?;

        writeln!(&mut code, "}}")?;
        writeln!(&mut code)?;
    }

    Ok(code)
}
