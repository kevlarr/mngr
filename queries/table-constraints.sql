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
select
    tbl.oid as table_oid,
    jsonb_object_agg(col_con.column, col_con.constraints) as "column_constraints!:ColumnConstraints"

from tbl

left join (
    select
        con.conkey[1] as "column",
        array_agg(jsonb_pretty(jsonb_build_object(
            'name', con.conname,
            'constraint_type', con.contype,
            'expression', pg_get_constraintdef(con.oid),
            'foreign_ref', case
                when con.confrelid = 0 then null
                else jsonb_build_object(
                    -- Need to cast oid to integer, since it gets cast to text by default
                    'oid', con.confrelid::integer,
                    'match_type', con.confmatchtype,
                    'columns', con.confkey
                )
            end
        ))) as constraints

    from tbl
    join pg_constraint con on con.conrelid = tbl.oid

    where array_length(con.conkey, 1) = 1

    group by con.conkey
) col_con on true

-- TODO: Surface table constraints
-- left join (
--     select
--         con.conkey as columns,
--         array_agg(jsonb_pretty(jsonb_build_object(
--             'name', con.conname,
--             'columns', con.conkey,
--             'constraint_type', con.contype,
--             'expression', pg_get_constraintdef(con.oid),
--             'foreign_ref', case
--                 when con.confrelid = 0 then null
--                 else jsonb_build_object(
--                     'oid', con.confrelid::integer,
--                     'match_type', con.confmatchtype,
--                     'columns', con.confkey
--                 )
--             end
--         ))) as constraints

--     from tbl
--     join pg_constraint con on con.conrelid = tbl.oid

--     where array_length(con.conkey, 1) > 1

--     group by con.conkey
-- ) tbl_con on true

group by tbl.oid
;
