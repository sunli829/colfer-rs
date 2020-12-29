#[derive(Debug, Eq, PartialEq)]
pub enum FieldType {
    Bool,
    U8,
    U16,
    U32,
    U64,
    I32,
    I64,
    F32,
    F64,
    Timestamp,
    Text,
    Binary,
    Struct(String),
    ArrayF32,
    ArrayF64,
    ArrayText,
    ArrayBinary,
    ArrayStruct(String),
}

#[derive(Debug, Eq, PartialEq)]
pub struct Field {
    pub name: String,
    pub ty: FieldType,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Colfer {
    pub package: String,
    pub structs: Vec<Struct>,
}

impl Colfer {
    pub fn need_box(&self, start: &str, ty: &str) -> bool {
        if let Some(s) = self.structs.iter().find(|s| s.name == start) {
            for field in &s.fields {
                if let FieldType::Struct(struct_name) = &field.ty {
                    if struct_name == start {
                        return true;
                    } else {
                        if self.need_box(&struct_name, ty) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}
