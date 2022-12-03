use maud::{html, Markup, Render};
use sqlx::{postgres::PgRow, Error as SqlError, Row};
use time::{macros::format_description, Date, PrimitiveDateTime};

use crate::state::Column;


#[derive(Copy, Clone)]
struct Days(usize);

impl Render for Days {
    fn render(&self) -> Markup {
        html! { (self.0) }
    }
}


#[derive(Copy, Clone)]
struct Seconds(usize);

impl Render for Seconds {
    fn render(&self) -> Markup {
        html! { (self.0) }
    }
}


#[derive(Default)]
pub struct DateAttributes {
    min: Option<Date>,
    max: Option<Date>,
    step: Option<Days>,
}

#[derive(Default)]
pub struct DateTimeAttributes {
    min: Option<PrimitiveDateTime>,
    max: Option<PrimitiveDateTime>,
    step: Option<Seconds>,
}

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
    Date(DateAttributes),
    DateTime(DateTimeAttributes),
    Number(NumberInputAttributes),
    Text(TextInputAttributes),
    TextArea(TextAreaAttributes),
}


pub struct Field {
    name: String,
    input_type: InputType,
    value: Option<String>,
}

impl Field {
    /*
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
    */

    fn value(&mut self, val: String) {
        self.value = Some(val);
    }
}

impl From<&Column> for Field {
    fn from(column: &Column) -> Self {
        let input_type = match column.data_type.as_ref() {
            "date" =>
                InputType::Date(DateAttributes::default()),
            "int4" | "int8" =>
                InputType::Number(NumberInputAttributes::default()),
            "text" =>
                InputType::TextArea(TextAreaAttributes::default()),
            "timestamptz" =>
                InputType::DateTime(DateTimeAttributes::default()),
            _ =>
                InputType::Text(TextInputAttributes::default()),
        };

        Self {
            name: column.name.to_owned(),
            value: None,
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
                InputType::Date(attrs) => {
                    @let format = format_description!("[year]-[month]-[day]");
                    @let min = attrs.min.map(|min| min.format(&format).unwrap());
                    @let max = attrs.max.map(|max| max.format(&format).unwrap());

                    input
                        id=(self.name)
                        name=(self.name)
                        type="date"
                        min=[min]
                        max=[max]
                        step=[attrs.step]
                        value=[&self.value]
                    {
                    }
                }
                InputType::DateTime(attrs) => {
                    @let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
                    @let min = attrs.min.map(|min| min.format(&format).unwrap());
                    @let max = attrs.max.map(|max| max.format(&format).unwrap());

                    // Haha this is hacky haha
                    @let value = self.value.as_ref().map(|v| v.split("+").next().unwrap().to_owned());

                    input
                        id=(self.name)
                        name=(self.name)
                        type="datetime-local"
                        min=[min]
                        max=[max]
                        step=[attrs.step]
                        value=[value]
                    {
                    }
                }
                InputType::Number(attrs) => {
                    input
                        id=(self.name)
                        name=(self.name)
                        type="number"
                        min=[attrs.min]
                        max=[attrs.max]
                        step=[attrs.step]
                        value=[&self.value]
                    {
                    }
                }
                InputType::Text(attrs) => {
                    input
                        id=(self.name)
                        name=(self.name)
                        type="text"
                        minlength=[attrs.minlength]
                        maxlength=[attrs.maxlength]
                        placeholder=[&attrs.placeholder]
                        value=[&self.value]
                    {
                    }
                }
                InputType::TextArea(attrs) => {
                    @let rows = attrs.rows.unwrap_or(1);

                    textarea
                        id=(self.name)
                        name=(self.name)
                        maxlength=[attrs.maxlength]
                        minlength=[attrs.minlength]
                        rows=(rows)
                    {
                        @if let Some(value) = &self.value {
                            (value)
                        }
                    }
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

    pub fn row(mut self, row: &PgRow) -> Self {
        for field in &mut self.fields {
            let value: String = row.try_get(field.name.as_str()).unwrap();

            field.value(value);
        }

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
