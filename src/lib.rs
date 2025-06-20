pub enum FormValue {
    Text(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Image(Vec<u8>),
}

pub struct FormInput {
    pub id: String,
    pub label: String,
    pub help: String,
    pub input_type: FormInputType,
}

pub struct SelectValue {
    pub id: String,
    pub label: String,
}

pub type SelectValues = Vec<SelectValue>;

pub enum FormInputType {
    Text,
    Number,
    CheckboxBoolean,
    CheckboxMultiple(SelectValues),
    Radio(SelectValues),
    Select(SelectValues),
    Signature,
    ImageFile,
}

pub trait ConnectionModel {
    fn set_form_value(&mut self, key: &str, value: FormValue);
}

#[derive(PartialEq, Debug)]
pub struct Connection {}

pub fn connect() -> Option<Connection> {
    None
}

#[cfg(test)]
mod tests {
    use super::connect;

    #[test]
    fn connection_fails() {
        let connection = connect();
        assert_eq!(connection, None);
    }
}
