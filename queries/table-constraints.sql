select
    conkey as "columns!:Vec<Position>",

    jsonb_build_object(

        -- This will exclude TRIGGER ('t') constraints since those are fired
        -- after a row insert, meaning it is hard to validate them beforehand...
        -- TODO: Is this accurate?
        'check', array_agg(jsonb_build_object(
            'name', conname,
            'expression', substring(
                expression
                from 9
                for length(expression) - 10
            )
        )) filter (where contype = 'c'),

        'exclusion', array_agg(jsonb_build_object(
            'name', conname,
            'expression', expression
        )) filter (where contype = 'x'),

        'foreign_key', array_agg(jsonb_build_object(
            'name', conname,
            'expression', expression,
            'foreign_ref', foreign_ref
        )) filter (where contype = 'f'),

        'primary_key', array_agg(jsonb_build_object(
            'name', conname
        )) filter (where contype = 'p'),

        'uniqueness', array_agg(jsonb_build_object(
            'name', conname
        )) filter (where contype = 'u')

    ) as "constraint_map!:Json<ConstraintMap>"

from (
    select
        conkey,
        conname,
        contype,
        pg_get_constraintdef(con.oid) as expression,

        -- Using nested rows is a royal pain in sqlx, so just convert to a json object
        case
        when confrelid = 0 then null
        else jsonb_build_object(
            'confkey', confkey,
            'confmatchtype', confmatchtype,
            'confrelid', confrelid::int
        )
        end as foreign_ref

    from pg_constraint con

    where
        array_length(conkey, 1) = 1 and
        conrelid = $1

    order by conkey
) con

group by conkey
order by conkey
