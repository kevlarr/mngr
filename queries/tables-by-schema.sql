select
    nspname as name,
    array_agg("table") as "tables!:Vec<SchemaTable>"

from (
    select
        n.nspname,
        jsonb_build_object(
          'name', c.relname,
          'oid', c.oid::integer
        ) as "table"

    from pg_class c
    join pg_namespace n on n.oid = c.relnamespace

    where
        -- Should this be optimized a bit..?
        concat(n.nspname, '.', c.relname) like any($1) and
        concat(n.nspname, '.', c.relname) not like any($2) and
        n.nspname = any(current_schemas(false)) and -- exclude implicit schemas
        c.relkind = 'r'

    order by n.nspname, c.relname
) q

group by nspname
;

