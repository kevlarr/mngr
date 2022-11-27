
use maud::{html, Markup, Render};
use sqlx::Error as SqlError;

use crate::state::Column;



#[derive(Default)]
pub struct NumberInputAttributes {
    min: Option<i64>,
    max: Option<i64>,
    step: Option<i64>,
}

#[derive(Default)]
pub struct TextInputAttributes {
    minlength: Option<i64>,
    maxlength: Option<i64>,
    placeholder: Option<String>,
}

#[derive(Default)]
pub struct TextAreaAttributes {
    minlength: Option<i64>,
    maxlength: Option<i64>,
    rows: Option<i64>,
}

pub enum InputType {
    Number(NumberInputAttributes),
    Text(TextInputAttributes),
    TextArea(TextAreaAttributes),
}


pub struct Field {
    name: String,
    input_type: InputType,
}

impl Field {
    pub fn number(mut self, callback: fn(&mut NumberInputAttributes)) -> Self {
        let mut attrs = NumberInputAttributes::default();
        callback(&mut attrs);

        self.input_type = InputType::Number(attrs);
        self
    }

    pub fn textarea(mut self, callback: fn(&mut TextAreaAttributes)) -> Self {
        let mut attrs = TextAreaAttributes::default();
        callback(&mut attrs);

        self.input_type = InputType::TextArea(attrs);
        self
    }
}

impl From<&Column> for Field {
    fn from(column: &Column) -> Self {
        let input_type = match column.meta.data_type.as_ref() {
            "bigint" | "integer" =>
                InputType::Number(NumberInputAttributes::default()),
            "text" =>
                InputType::TextArea(TextAreaAttributes::default()),
            _ =>
                InputType::Text(TextInputAttributes::default()),
        };

        Self {
            name: column.name.to_owned(),
            input_type,
        }
    }
}

impl Render for Field {
    fn render(&self) -> Markup {
        // TODO: id different from name
        
        html! {
            label for=(self.name) { (self.name) }

            @match &self.input_type {
                InputType::Number(attrs) => {
                    input
                        id=(self.name)
                        name=(self.name)
                        type="number"
                        min=[attrs.min]
                        max=[attrs.max]
                        step=[attrs.step]
                        {}
                }
                InputType::Text(attrs) => {
                    input
                        id=(self.name)
                        name=(self.name)
                        type="text"
                        minlength=[attrs.minlength]
                        maxlength=[attrs.maxlength]
                        placeholder=[&attrs.placeholder]
                        {}
                }
                InputType::TextArea(attrs) => {
                    @let rows = attrs.rows.unwrap_or(1);

                    textarea
                        id=(self.name)
                        name=(self.name)
                        maxlength=[attrs.maxlength]
                        minlength=[attrs.minlength]
                        rows=(rows)
                        {}
                }
            }
        }
    }
}



#[derive(Default)]
pub struct Form {
    action: Option<String>,
    error: Option<SqlError>,
    fields: Vec<Field>,
    method: Option<String>,
    submit_text: Option<String>,
}

impl Form {
    pub fn action(mut self, action: &str) -> Self {
        self.action = Some(action.to_owned());
        self
    }

    pub fn method(mut self, method: &str) -> Self {
        self.method = Some(method.to_owned());
        self
    }

    pub fn error(mut self, error: SqlError) -> Self {
        self.error = Some(error);
        self
    }

    fn add_field(&mut self, field: Field) {
        self.fields.push(field);
    }
}

impl From<&[Column]> for Form {
    fn from(columns: &[Column]) -> Self {
        let mut form = Self::default();

        for column in columns.iter() {
            if column.always_generated() { continue; }

            form.add_field(Field::from(column));
        }

        form
    }
}

impl Render for Form {
    fn render(&self) -> Markup {
        let submit_text = self.submit_text.as_deref().unwrap_or("Submit");

        html! {
            c-form {
                form method=[&self.method] action=[&self.action] {
                    @for field in &self.fields {
                        c-form-field { (field) }
                    }
                    c-form-controls {
                        button type="submit" { (submit_text) }
                    }
                    @if let Some(error) = &self.error {
                        output class="error" {
                            pre { (format!("{error:#?}")) }
                        }
                    }
                }
            }
        }
    }
}
