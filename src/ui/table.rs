
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
            data_type: column.data_type.clone(),
            name: column.name.clone(),
        }
    }
}

#[derive(Default)]
pub struct Table {
    schema_name: String,
    table_name: String,
    columns: Vec<TableColumn>,
    rows: Vec<PgRow>,
}

impl Table {
    pub fn new(
        schema_name: &str,
        table_name: &str,
        columns: &[Column],
        rows: Vec<PgRow>,
    ) -> Self {
        let columns = columns.iter()
            .map(|c| TableColumn::from(c))
            .collect();

        Self {
            schema_name: schema_name.to_owned(),
            table_name: table_name.to_owned(),
            columns,
            rows,
         }
    }

    fn render_row(&self, columns: &[TableColumn], row: &PgRow) -> Markup {
        html! {
            @let record_id: String = row.try_get("id").unwrap();

            tr data-schema=(self.schema_name) data-table=(self.table_name) data-record=(record_id) {
                @for column in columns {
                    @let col_name: &str = column.name.as_ref();
                    @let value: String = row.try_get(col_name).unwrap();

                    td class=(&column.data_type) {
                        @match column.data_type.as_ref() {
                            "ltree" => {
                                code { (value) }
                            }
                            _ => {
                                (value)
                            }
                        }
                    }
                }
            }
        }
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
                                th class=(column.data_type) data-column=(column.name) { (column.name) }
                            }
                        }
                    }
                    tbody {
                        @for row in &self.rows {
                            (self.render_row(self.columns.as_slice(), &row))
                        }
                    }
                    caption {
                        "Double-click any row to edit"
                    }
                }
            }
        }
    }
}
