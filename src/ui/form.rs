use std::collections::HashMap;

use maud::{html, Markup, Render};
use sqlx::{
    postgres::PgRow,
    types::Json,
    Error as SqlError,
    Row,
};
use time::{macros::format_description, Date, PrimitiveDateTime};

use crate::{
    db::{Column, Constraint, Position, Table},
    ui::utils::render_markdown,
};

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

// TODO: These can probably ALL go away since there isn't going to be
// any attempt to parse constraints in order to populate any of these values,
// and there would be no concept of "step" anyway.
//
// Or maybe config should allow for populating any of these..?
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

            @if let Some(comment) = &self.column.description {
                .description {
                    (render_markdown(comment))
                }
            }
        }
    }
}

struct PartitionedConstraints<'a, 'b> {
    column: HashMap<Position, &'a Vec<Json<Constraint>>>,
    table: HashMap<&'b Vec<Position>, &'b Vec<Json<Constraint>>>,
}

pub struct Form<'row, 'tbl> {
    action: Option<String>,
    error: Option<SqlError>,
    method: Option<String>,
    row: Option<&'row PgRow>,
    submit_text: Option<String>,
    table: &'tbl Table,
}

impl<'row, 'tbl> Form<'row, 'tbl> {
    pub fn new(table: &'tbl Table) -> Self {
        Self {
            action: None,
            error: None,
            method: None,
            row: None,
            submit_text: None,
            table,
        }
    }

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

    pub fn row(mut self, row: &'row PgRow) -> Self {
        self.row = Some(row);
        self
    }

    fn partition_constraints(&self) -> PartitionedConstraints {
        let mut column = HashMap::new();
        let mut table = HashMap::new();

        for constraint_set in &self.table.constraints {
            match constraint_set.columns.as_slice() {
                [col] => {
                    column.insert(*col, &constraint_set.constraints);
                },
                _ => {
                    table.insert(&constraint_set.columns, &constraint_set.constraints);
                },
            }
        }

        PartitionedConstraints { column, table }
    }
}

impl<'a, 'b> Render for Form<'a, 'b> {
    fn render(&self) -> Markup {
        let submit_text = self.submit_text.as_deref().unwrap_or("Submit");

        let mut fields = Vec::new();

        for column in self.table.columns.iter() {
            if column.always_generated() { continue; }

            let mut field = Field::from(column);

            if let Some(row) = &self.row {
                let value: String = row.try_get(column.name.as_str()).unwrap();
                field.value(value);
            }

            fields.push(field);
        }

        let constraints = self.partition_constraints();

        html! {
            c-form {
                form method=[&self.method] action=[&self.action] {
                    @for field in &fields {
                        c-form-field { (field) }
                        @if let Some(cons) = constraints.column.get(&field.column.position) {
                            @for con in cons.iter() {
                                pre { (format!("{:?}", con)) }
                            }
                        }
                    }

                    @if constraints.table.len() > 0 {
                        .bold { "Table Constraints" }
                        pre { (format!("{:?}", constraints.table)) }
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
