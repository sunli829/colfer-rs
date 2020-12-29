use std::fmt::Write;

use crate::ast::{Colfer, FieldType};

pub fn generate(colfer: &Colfer) -> String {
    let mut code = String::new();

    writeln!(
        &mut code,
        "#![allow(unused_variables, unused_assignments, unused_mut)]"
    )
    .unwrap();
    writeln!(&mut code).unwrap();
    writeln!(&mut code, "use std::io::{{Write, Read, Result}};").unwrap();
    writeln!(&mut code).unwrap();
    writeln!(&mut code, "use colfer::{{Message, Type, DateTime}};").unwrap();
    writeln!(&mut code).unwrap();

    for s in &colfer.structs {
        writeln!(&mut code, "#[derive(Default, Clone, Debug, PartialEq)]").unwrap();
        writeln!(&mut code, "pub struct {} {{", s.name).unwrap();

        for f in &s.fields {
            write!(&mut code, "\tpub {}: ", f.name).unwrap();

            match &f.ty {
                FieldType::Bool => write!(&mut code, "bool").unwrap(),
                FieldType::U8 => write!(&mut code, "u8").unwrap(),
                FieldType::U16 => write!(&mut code, "u16").unwrap(),
                FieldType::U32 => write!(&mut code, "u32").unwrap(),
                FieldType::U64 => write!(&mut code, "u64").unwrap(),
                FieldType::I32 => write!(&mut code, "i32").unwrap(),
                FieldType::I64 => write!(&mut code, "i64").unwrap(),
                FieldType::F32 => write!(&mut code, "f32").unwrap(),
                FieldType::F64 => write!(&mut code, "f64").unwrap(),
                FieldType::Timestamp => write!(&mut code, "DateTime").unwrap(),
                FieldType::Text => write!(&mut code, "String").unwrap(),
                FieldType::Binary => write!(&mut code, "Vec<u8>").unwrap(),
                FieldType::Struct(name) => {
                    if colfer.need_box(&s.name, &name) {
                        write!(&mut code, "Option<Box<{}>>", name).unwrap();
                    } else {
                        write!(&mut code, "Option<{}>", name).unwrap();
                    }
                }
                FieldType::ArrayF32 => write!(&mut code, "Vec<f32>").unwrap(),
                FieldType::ArrayF64 => write!(&mut code, "Vec<f64>").unwrap(),
                FieldType::ArrayText => write!(&mut code, "Vec<String>").unwrap(),
                FieldType::ArrayBinary => write!(&mut code, "Vec<Vec<u8>>").unwrap(),
                FieldType::ArrayStruct(name) => write!(&mut code, "Vec<{}>", name).unwrap(),
            }

            writeln!(&mut code, ",").unwrap();
        }

        writeln!(&mut code, "}}").unwrap();

        writeln!(&mut code).unwrap();
        writeln!(&mut code, "impl Message for {} {{", s.name).unwrap();

        writeln!(
            &mut code,
            "\tfn encode<W: Write>(&self, w: &mut W) -> Result<()> {{"
        )
        .unwrap();
        for (idx, f) in s.fields.iter().enumerate() {
            match &f.ty {
                FieldType::Struct(_) => writeln!(
                    &mut code,
                    "\t\tcolfer::encode_message(w, {}, self.{}.as_deref())?;",
                    idx, f.name
                )
                .unwrap(),
                FieldType::ArrayStruct(_) => writeln!(
                    &mut code,
                    "\t\tcolfer::encode_messages(w, {}, &self.{})?;",
                    idx, f.name
                )
                .unwrap(),
                _ => writeln!(&mut code, "\t\tself.{}.encode(w, {})?;", f.name, idx).unwrap(),
            }
        }
        writeln!(&mut code).unwrap();
        writeln!(&mut code, "\t\tOk(())\n\t}}").unwrap();
        writeln!(&mut code).unwrap();

        writeln!(
            &mut code,
            "\tfn decode<R: Read>(r: &mut R) -> Result<Self> {{"
        )
        .unwrap();
        writeln!(&mut code, "\t\tlet mut obj = Self::default();").unwrap();
        writeln!(
            &mut code,
            "\t\tlet (mut id, mut flag) = colfer::read_header(r)?;"
        )
        .unwrap();
        for (idx, f) in s.fields.iter().enumerate() {
            writeln!(&mut code, "\t\tif id == {} {{", idx).unwrap();
            match &f.ty {
                FieldType::Struct(_) => writeln!(
                    &mut code,
                    "\t\t\tobj.{} = colfer::decode_message(r)?;",
                    f.name,
                )
                .unwrap(),
                FieldType::ArrayStruct(_) => writeln!(
                    &mut code,
                    "\t\t\tobj.{} = colfer::decode_messages(r)?;",
                    f.name,
                )
                .unwrap(),
                _ => writeln!(&mut code, "\t\t\tobj.{} = Type::decode(r, flag)?;", f.name).unwrap(),
            }
            if idx < s.fields.len() - 1 {
                writeln!(&mut code, "\t\t\tlet next = colfer::read_header(r)?;").unwrap();
                writeln!(&mut code, "\t\t\tid = next.0;").unwrap();
                writeln!(&mut code, "\t\t\tflag = next.1;").unwrap();
            }
            writeln!(&mut code, "\t\t}}").unwrap();
        }
        writeln!(&mut code).unwrap();

        writeln!(&mut code, "\t\tOk(obj)\n\t}}").unwrap();
        writeln!(&mut code, "}}").unwrap();
        writeln!(&mut code).unwrap();
    }

    code
}
