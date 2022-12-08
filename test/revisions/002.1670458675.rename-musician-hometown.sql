-- Revision: rename-musician-hometown
--
-- Add description here

begin;

alter table musician rename hometown to hometown_id;

commit;
