select
    conkey as "columns!:Vec<Position>",
    array_agg(jsonb_build_object(
      'name', conname,
      'constraint_type', contype,
      'expression', expression,
      'foreign_ref', foreign_ref
    )) as "constraints!:Vec<Json<Constraint>>"

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
;

