# Testing


## Setup

Manager can be tested against any existing database, but a sample schema
can be easily loaded via the provided `jrny` revisions, provided the `ltree`
extension is enabled and in the search path as follows:

```sql
create schema ext;
create extension ltree with schema ext;
alter database mngr set search_path = "$user", public, ext;
```

Then create a `jrny-env.toml` file pointing to the test database and run `jrny embark`.
