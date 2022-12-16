-- Revision: more-comments
--
-- Add description here

begin;

-- Add SQL here
comment on column genre.path is 'The categorical `ltree` path, eg. `rock.classic`';
comment on column genre.name is 'The user-friendly display name for the genre, eg. *"Classic Rock"*';

commit;
