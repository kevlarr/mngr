
use maud::{html, Markup, Render};
use sqlx::{Row, postgres::PgRow};

use crate::state::Column;

pub struct TableColumn {
    data_type: String,
    name: String,
}

impl From<&Column> for TableColumn {
    fn from(column: &Column) -> Self {
        Self {
            data_type: column.meta.data_type.clone(),
            name: column.name.clone(),
        }
    }
}

#[derive(Default)]
pub struct Table {
    columns: Vec<TableColumn>,
    rows: Vec<PgRow>,
}

impl Table {
    pub fn new(columns: &[Column], rows: Vec<PgRow>) -> Self {
        let columns = columns.iter()
            .map(|c| TableColumn::from(c))
            .collect();

        Self { columns, rows }
    }
}

impl Render for Table {
    fn render(&self) -> Markup {
        // Places the table in a wrapper so that the wrapping container can be used
        // in flex containers, with overflow working as expected, etc.
        html! {
            c-table {
                table {
                    thead {
                        tr {
                            @for column in &self.columns {
                                th class=(column.data_type) { (column.name) }
                            }
                        }
                    }
                    tbody {
                        @for row in &self.rows {
                            (render_row(self.columns.as_slice(), &row))
                        }
                    }
                }
            }
        }
    }
}

fn render_row(columns: &[TableColumn], row: &PgRow) -> Markup {
    html! {
        tr {
            @for column in columns {
                @let col_name: &str = column.name.as_ref();
                @let value: String = row.try_get(col_name).unwrap();

                td class=(&column.data_type) { (value) }
            }
        }
    }
}
