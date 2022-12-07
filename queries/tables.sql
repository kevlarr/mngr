select
    schema_name as name,
    array_agg(jsonb_pretty(jsonb_build_object(
        'name', table_name,
        'columns', columns
    ))) as "tables!:Vec<Table>"

from (
    select
        schema_name,
        table_name,
        array_agg(jsonb_build_object(
            'name', column_name,
            'data_type', type_name,
            'position', position,
            'nullable', nullable,
            'identity', identity,
            'generated', generated,
            'expression', expression
        )) as columns

    from (
        select
            n.nspname as schema_name,
            c.relname as table_name,
            a.attname as column_name,
            t.typname as type_name,
            a.attnum  as position,
            not a.attnotnull as nullable,

            case
            when a.attidentity = 'a' then 'always'
            when a.attidentity = 'd' then 'default'
            end as identity,

            case
            when a.attgenerated = 's' then 'stored'
            end as generated,

            pg_get_expr(ad.adbin, ad.adrelid) as expression

        from pg_attribute     a
        join pg_class         c on c.oid = a.attrelid
        join pg_namespace     n on n.oid = c.relnamespace
        join pg_type          t on t.oid = a.atttypid
        left join pg_attrdef ad on ad.adrelid = c.oid and ad.adnum = a.attnum

        where
            -- Does this need to be optimized a bit..?
            concat(n.nspname, '.', c.relname) like any($1) and
            concat(n.nspname, '.', c.relname) not like any($2) and
            c.relkind = 'r' and
            not a.attisdropped  and
            a.attnum > 0

        order by n.nspname, c.relname, a.attnum
    ) q2

    group by schema_name, table_name

    order by schema_name, table_name

) q1

group by schema_name

order by schema_name
;

