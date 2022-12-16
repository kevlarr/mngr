use crate::{
    db::Column,
    ui::utils::render_markdown,
};
use maud::{html, Markup, Render};
use sqlx::{postgres::PgRow, Error as SqlError, Row};
use time::{macros::format_description, Date, PrimitiveDateTime};

#[derive(Copy, Clone, PartialEq)]
struct Days(usize);

impl Render for Days {
    fn render(&self) -> Markup {
        html! { (self.0) }
    }
}

#[derive(Copy, Clone, PartialEq)]
struct Seconds(usize);

impl Render for Seconds {
    fn render(&self) -> Markup {
        html! { (self.0) }
    }
}

#[derive(Default, PartialEq)]
pub struct DateAttributes {
    min: Option<Date>,
    max: Option<Date>,
    step: Option<Days>,
}

#[derive(Default, PartialEq)]
pub struct DateTimeAttributes {
    min: Option<PrimitiveDateTime>,
    max: Option<PrimitiveDateTime>,
    step: Option<Seconds>,
}

#[derive(Default, PartialEq)]
pub struct NumberInputAttributes {
    min: Option<i64>,
    max: Option<i64>,
    step: Option<i64>,
}

#[derive(Default, PartialEq)]
pub struct TextInputAttributes {
    minlength: Option<i64>,
    maxlength: Option<i64>,
    placeholder: Option<String>,
}

#[derive(Default, PartialEq)]
pub struct TextAreaAttributes {
    minlength: Option<i64>,
    maxlength: Option<i64>,
    rows: Option<i64>,
}

#[derive(PartialEq)]
pub enum InputType {
    Boolean,
    Date(DateAttributes),
    DateTime(DateTimeAttributes),
    Number(NumberInputAttributes),
    Text(TextInputAttributes),
    TextArea(TextAreaAttributes),
}


pub struct Field<'a> {
    column: &'a Column,
    input_type: InputType,
    value: Option<String>,
}

impl<'a> Field<'a> {
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

impl<'a, 'b: 'a> From<&'b Column> for Field<'a> {
    fn from(column: &'b Column) -> Self {
        // Unless these are ever individually-configured, this could simply
        // be moved to render
        let input_type = match column.data_type.as_ref() {
            "bool" =>
                InputType::Boolean,
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
            column,
            input_type,
            value: None,
        }
    }
}

impl<'a> Render for Field<'a> {
    fn render(&self) -> Markup {
        // TODO: id different from name
        let id = &self.column.name;
        let data_type = &self.column.data_type;
        let required = self.column.required() && self.input_type != InputType::Boolean;

        html! {
            label.required[required] for=(id) { (id) }

            @match &self.input_type {
                InputType::Boolean => {
                    @let checked = self.value.as_deref() == Some("true");

                    input
                        id=(id)
                        name=(id)
                        type="checkbox"
                        class=(data_type)
                        checked[checked]
                    {
                    }
                }
                InputType::Date(attrs) => {
                    @let format = format_description!("[year]-[month]-[day]");
                    @let min = attrs.min.map(|min| min.format(&format).unwrap());
                    @let max = attrs.max.map(|max| max.format(&format).unwrap());

                    input
                        id=(id)
                        name=(id)
                        type="date"
                        class=(data_type)
                        min=[min]
                        max=[max]
                        step=[attrs.step]
                        value=[&self.value]
                        required[required]
                    {
                    }
                }
                InputType::DateTime(attrs) => {
                    @let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
                    @let min = attrs.min.map(|min| min.format(&format).unwrap());
                    @let max = attrs.max.map(|max| max.format(&format).unwrap());

                    // Haha this is so hacky haha
                    //
                    // TODO: And it doesn't always work because it can still include milliseconds,
                    // which will not populate the input
                    @let value = self.value.as_ref().map(|v| v.split("+").next().unwrap().to_owned());

                    input
                        id=(id)
                        name=(id)
                        type="datetime-local"
                        class=(data_type)
                        min=[min]
                        max=[max]
                        step=[attrs.step]
                        value=[value]
                        required[required]
                    {
                    }
                }
                InputType::Number(attrs) => {
                    input
                        id=(id)
                        name=(id)
                        type="number"
                        class=(data_type)
                        min=[attrs.min]
                        max=[attrs.max]
                        step=[attrs.step]
                        value=[&self.value]
                        required[required]
                    {
                    }
                }
                InputType::Text(attrs) => {
                    input
                        id=(id)
                        name=(id)
                        type="text"
                        class=(data_type)
                        minlength=[attrs.minlength]
                        maxlength=[attrs.maxlength]
                        placeholder=[&attrs.placeholder]
                        value=[&self.value]
                        required[required]
                    {
                    }
                }
                InputType::TextArea(attrs) => {
                    @let rows = attrs.rows.unwrap_or(1);

                    textarea
                        id=(id)
                        name=(id)
                        class=(data_type)
                        maxlength=[attrs.maxlength]
                        minlength=[attrs.minlength]
                        rows=(rows)
                        required[required]
                    {
                        @if let Some(value) = &self.value {
                            (value)
                        }
                    }
                }
            }

            @if let Some(comment) = &self.column.comment {
                .description {
                    (render_markdown(comment))
                }
            }
        }
    }
}


#[derive(Default)]
pub struct Form<'a> {
    action: Option<String>,
    error: Option<SqlError>,
    fields: Vec<Field<'a>>,
    method: Option<String>,
    submit_text: Option<String>,
}

impl<'a> Form<'a> {
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
            let value: String = row.try_get(field.column.name.as_str()).unwrap();

            field.value(value);
        }

        self
    }

    fn add_field(&mut self, field: Field<'a>) {
        self.fields.push(field);
    }
}

impl<'a, 'b: 'a> From<&'b [Column]> for Form<'a> {
    fn from(columns: &'b [Column]) -> Self {
        let mut form = Self::default();

        for column in columns.iter() {
            if column.always_generated() { continue; }

            form.add_field(Field::from(column));
        }

        form
    }
}

impl<'a> Render for Form<'a> {
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
