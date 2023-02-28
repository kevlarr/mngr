select
    cls.oid,
    cls.relname as "name",
    nsp.nspname as "schema",
    obj_description(cls.oid, 'pg_class') as "description"

from pg_class     cls
join pg_namespace nsp on nsp.oid = cls.relnamespace

where
    cls.relkind = 'r' and cls.oid = $1 and
    -- Verify that the table should be accessible based on the config file
    concat(nsp.nspname, '.', cls.relname)     like any($2) and
    concat(nsp.nspname, '.', cls.relname) not like any($3) and
    -- Verify that the table should be accessible based on the search path,
    -- excluding implicit tables (eg. pg_catalog)
    nsp.nspname = any(current_schemas(false))
