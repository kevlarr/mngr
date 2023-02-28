use crate::db;
use maud::{html, Markup, Render};
use sqlx::{Row, postgres::PgRow};

pub struct TableColumn {
    data_type: String,
    name: String,
}

impl From<&db::Column> for TableColumn {
    fn from(column: &db::Column) -> Self {
        Self {
            data_type: column.data_type.clone(),
            name: column.name.clone(),
        }
    }
}

pub struct Table<'a> {
    table: &'a db::Table,
    columns: Vec<TableColumn>,
    rows: Vec<PgRow>,
}

impl<'a, 'b: 'a> Table<'a> {
    pub fn new(table: &'b db::Table, rows: Vec<PgRow>) -> Self {
        let columns = table.columns.iter()
            .map(|c| TableColumn::from(c))
            .collect();

        Self {
            table,
            columns,
            rows,
         }
    }

    fn render_row(&self, columns: &[TableColumn], row: &PgRow) -> Markup {
        html! {
            @let record_id: String = row.try_get("id").unwrap();

            tr data-table-oid=(self.table.meta.oid.0) data-record-id=(record_id) {
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

impl<'a> Render for Table<'a> {
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
