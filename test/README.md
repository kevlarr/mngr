# Testing


## Setup

Manager can be tested against any existing database, but a sample schema
can be easily loaded via the provided `jrny` revisions, provided the `ltree`
extension is enabled as follows:

```sql
create schema ext_ltree;
create extension ltree with schema ext_ltree;
```

Then create a `jrny-env.toml` file pointing to the test database and run `jrny embark`.
