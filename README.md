# Manager

Manager intends to provide basic but incredibly convenient CRUD on existing database tables.

Good for people who...

* Prefer constraints and validations (and even object descriptions) to live in the database
* Want to expose a simple, intuitive CRUD UI with zero code - and zero alterations to the database
* Need a basic and immediate dashboard as a placeholder for a more advanced, custom-written one later

Not good for people who...

* Prefer code-level validations
* Want advanced functionality, because this be basic

## Motivation

So many options and none seemed to fit..

Wanted a simple-to-spin-up, compiled application that I could just put in front of a database and
have it 'just work' automatically, without any alterations to the database schema.
Directus, Piccolo, etc. all

Wanted to support wide variety of data types, even custom ones - Directus can't even
support `point` types without showing `[Object object]` in a text field.

## Goals

- Somewhat opinionated yet configurable
- Require zero alterations to the database (looking at you, Directus.. just wow)
- Handle unknown data types gracefully; ie. anything that can be cast to & from `text` type
- Make foreign key assignments easier; eg. type-ahead search to find a record, rather than remembering a primary key


## Running

```sh
# The `--no-auth` flag is required, even though authentication is not yet built in
$ mngr -c 'postgres://user:password@host:port/dbname' --no-auth
```

While developing, use `cargo watch` to automatically re-check and re-build on file changes,
clearing the terminal screen on each reload:

```sh
$ cargo install cargo-watch

# To run admin app
$ DATABASE_URL="..." STATIC_PATH="static/" cargo watch --clear --exec 'run'
```
