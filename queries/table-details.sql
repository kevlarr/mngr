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

    array_agg(jsonb_build_object(
        'data_type', typname,
        'expression', pg_get_expr(adbin, adrelid),
        'generated', case
            when attgenerated = 's' then 'stored'
        end,
        'identity', case
            when attidentity = 'a' then 'always'
            when attidentity = 'd' then 'default'
        end,
        'name', attname,
        'nullable', not attnotnull,
        'position', attnum
    )) as "columns!:Vec<Column>"

from tbl
join (
    select
        att.attname,
        att.attnum,
        att.attnotnull,
        att.attidentity,
        att.attgenerated,
        typ.typname,
        def.adbin,
        def.adrelid

    from      tbl
    join      pg_attribute att on att.attrelid = tbl.oid
    join      pg_type      typ on typ.oid      = att.atttypid
    left join pg_attrdef   def on def.adrelid  = tbl.oid and def.adnum = att.attnum

    where
        not att.attisdropped and
        att.attnum > 0

    order by att.attnum
) q2 on true

group by
    tbl.oid,
    tbl.relname,
    tbl.nspname
;

