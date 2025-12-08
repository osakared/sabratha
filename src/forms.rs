use async_graphql::*;
use turbosql::{Turbosql, select};
use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize, Enum, Copy, Clone, Eq, PartialEq)]
pub enum FormFieldType {
    Text,
    Number,
    CheckboxBoolean,
    CheckboxMultiple,
    Radio,
    Select,
    Signature,
    ImageFile,
}

impl Default for FormFieldType {
    fn default() -> Self { Self::Text }
}

#[derive(Turbosql, SimpleObject, InputObject, Clone)]
#[graphql(input_name = "FormFieldInput")]
pub struct FormField {
    // Only for local sqlite db so shouldn't be exposed to clients
    #[graphql(skip)]
    pub rowid: Option<i64>,
    // Globally unique id to facilitate decentralization
    pub id: String,
    // Effective foreign key (not using a join)
    pub form_id: String,
    pub label: String,
    pub help: String,
    pub input_type: FormFieldType,
    // Not making this algebraic subtype of enum due to limitations of gql
    pub option_values: Vec<String>,
}

impl Default for FormField {
    fn default() -> Self {
        Self {
            rowid: None,
            id: Uuid::new_v4().to_string(),
            form_id: String::default(),
            label: String::default(),
            help: String::default(),
            input_type: FormFieldType::default(),
            option_values: vec![],
        }
    }
}

impl FormField {
    /// Creates a `FormField` with `parent` as parent, tries to insert into db and returns result or error
    pub fn create(form_id:&String) -> Result<Self, turbosql::Error> {
        let mut form_field = Self::default();
        form_field.form_id = form_id.clone();
        match form_field.insert() {
            Ok(id) => select!(FormField "WHERE rowid = " id),
            Err(e) => Err(e)
        }
    }

    /// Tries to find a single `Form` with given id
    pub fn find(id:&String) -> Result<Self, turbosql::Error> {
        select!(FormField "where id = " id)
    }

    pub fn create_or_update(form_id:&String, form_field:&Self) -> Result<Self, turbosql::Error> {
        let mut new_form = match Self::find(&form_id) {
            Ok(mut f) => {
                f.form_id = form_id.clone();
                f
            },
            Err(_) => Self::create(&form_id)?
        };

        new_form.label = form_field.label.clone();
        new_form.help = form_field.help.clone();
        new_form.input_type = form_field.input_type;
        new_form.option_values = form_field.option_values.clone();
        
        match new_form.update() {
            Ok(_) => Ok(new_form),
            Err(e) => Err(e)
        }
    }
}

#[derive(Turbosql, SimpleObject, InputObject, Clone)]
#[graphql(input_name = "FormInput")]
pub struct Form {
    // Only for local sqlite db so shouldn't be exposed to clients
    #[graphql(skip)]
    pub rowid: Option<i64>,
    // Globally unique id to facilitate decentralization
    pub id: String,
    pub title: String,
}

impl Default for Form {
    fn default() -> Self {
        Self {
            rowid: None,
            id: Uuid::new_v4().to_string(),
            title: String::default(),
        }
    }
}

#[ComplexObject]
impl Form {
    async fn fields(&self) -> Vec<FormField> {
        self.form_fields()
    }
}

impl Form {
    /// Creates a `Form`, tries to insert into db and returns result or error
    pub fn create() -> Result<Self, turbosql::Error> {
        let form = Self::default();
        match form.insert() {
            Ok(id) => select!(Form "WHERE rowid = " id),
            Err(e) => Err(e)
        }
    }

    /// Gets child `FormField`s
    pub fn form_fields(&self) -> Vec<FormField> {
        select!(Vec<FormField> "WHERE form_id = ?", self.id).unwrap_or(vec![])
    }

    /// Gets `Vec` of `Form`s
    pub fn query() -> Vec<Self> {
        select!(Vec<Form>).unwrap_or(vec![])
    }

    /// Tries to find a single `Form` with given id
    pub fn find(id:&String) -> Result<Self, turbosql::Error> {
        select!(Form "where id = " id)
    }

    /// Finds a `Form` based on `form.id` and if not present, creates a new one, then updates from contents of `form`
    pub fn create_or_update(form:&Self) -> Result<Self, turbosql::Error> {
        let mut new_form = match Self::find(&form.id) {
            Ok(f) => f,
            Err(_) => Self::create()?
        };

        new_form.title = form.title.clone();

        for form_field in form.form_fields() {
            FormField::create_or_update(&new_form.id, &form_field)?;
        }
        
        match new_form.update() {
            Ok(_) => Ok(new_form),
            Err(e) => Err(e)
        }
    }
}
