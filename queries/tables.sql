select
    q.schema_name as "name!",
    array_agg(q.table_json) as "tables!:Vec<Table>"

from (
    select
        t.table_schema as schema_name,
        json_build_object(
          'name', t.table_name,
          'meta', to_json(t),
          'columns', array_agg(
            json_build_object(
              'name', c.column_name,
              'meta', to_json(c)
            )
          )
        ) as table_json

    from information_schema.tables t
    join (
      select *
      from information_schema.columns

      -- Filtering columns by schema/table works as well as filtering the tables table itself
      -- where concat(table_schema, '.', table_name) = any($1)
      where table_schema = 'public'

      order by
        table_schema,
        table_name,
        ordinal_position

    ) c using (table_schema, table_name)

    group by
        t.*,
        t.table_schema,
        t.table_name
) q

group by q.schema_name
;
