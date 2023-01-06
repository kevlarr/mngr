-- Selects a single table matching the given OID, provided that it
-- is also an 'available' table
with tbl as (
    select
        cls.oid,
        cls.relname,
        nsp.nspname

    from pg_class     cls
    join pg_namespace nsp on nsp.oid = cls.relnamespace

    where
        concat(nsp.nspname, '.', cls.relname)     like any($1) and
        concat(nsp.nspname, '.', cls.relname) not like any($2) and
        nsp.nspname = any(current_schemas(false))              and
        cls.relkind = 'r'                                      and
        cls.oid = $3

    limit 1 -- Is there any reason to include this?
)

-- Select all necessary column, type, and 'definition' information
-- to add to the table details selected above
select
    tbl.oid,
    tbl.relname as "name",
    tbl.nspname as "schema",
    obj_description(tbl.oid, 'pg_class') as comment,
    col.columns as "columns!:Vec<Column>"

from tbl

join (
    select
        array_agg(jsonb_build_object(
            'comment', col_description(tbl.oid, att.attnum),
            'data_type', typ.typname,
            'expression', pg_get_expr(def.adbin, def.adrelid),
            'generated', case att.attgenerated
                when '' then null
                else att.attgenerated
            end,
            'identity', case att.attidentity
                when '' then null
                else att.attidentity
            end,
            'name', att.attname,
            'nullable', not att.attnotnull,
            'position', att.attnum
        ) order by att.attnum) as columns

    from      tbl
    join      pg_attribute att on att.attrelid = tbl.oid
    join      pg_type      typ on att.atttypid = typ.oid
    left join pg_attrdef   def on att.attrelid = def.adrelid and def.adnum = att.attnum

    where
        not att.attisdropped and
        att.attnum > 0
) col on true
;
