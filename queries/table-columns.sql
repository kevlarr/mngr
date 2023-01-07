-- Select all necessary column, type, and 'definition' information
select
        typ.typname as "data_type",
        col_description(att.attrelid, att.attnum) as "description",
        pg_get_expr(def.adbin, def.adrelid) as "expression",
        case att.attgenerated when '' then null else att.attgenerated end as "generated:Generated",
        case att.attidentity  when '' then null else att.attidentity  end as "identity:Identity",
        att.attname as "name",
        not att.attnotnull as "nullable!",
        att.attnum as "position"

from      pg_attribute att
join      pg_type      typ on att.atttypid = typ.oid
left join pg_attrdef   def on att.attrelid = def.adrelid and def.adnum = att.attnum

where
    att.attrelid = $1 and
    not att.attisdropped and
    att.attnum > 0

order by
    att.attnum
;
