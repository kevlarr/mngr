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
    col.columns as "columns!:Vec<Column>",
    con.constraints as "constraints!:Vec<Constraint>"

from tbl

join (
    select
        array_agg(jsonb_build_object(
            'data_type', typ.typname,
            'expression', pg_get_expr(def.adbin, def.adrelid),
            'generated', case att.attgenerated
                when 's' then 'stored'
            end,
            'identity', case att.attidentity
                when 'a' then 'always'
                when 'd' then 'default'
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

join (
    select
        array_agg(jsonb_build_object(
            'name', con.conname,
            'columns', con.conkey,
            'constraint_type', case con.contype
                when 'c' then 'check'
                when 'f' then 'foreign_key'
                when 'p' then 'primary_key'
                when 'u' then 'unique'
                when 't' then 'constraint_trigger'
                when 'x' then 'exclusion'
            end,
            'expression', pg_get_constraintdef(con.oid),
            'foreign_ref', case
                when con.confrelid = 0 then null
                else jsonb_build_object(
                    -- An oid type is coerced to text by default when building a jsonb object
                    'oid', con.confrelid::integer,
                    'match_type', case con.confmatchtype
                        when 'f' then 'full'
                        when 'p' then 'partial'
                        when 's' then 'simple'
                    end,
                    'columns', con.confkey
                )
            end
        )) as constraints

    from tbl
    join pg_constraint con on con.conrelid = tbl.oid
) con on true
;
