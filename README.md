# Manager

Manager intends to provide basic but incredibly convenient CRUD on existing database tables.

Good for people who...

* Prefer constraints and validations (and even object descriptions) to live in the database
* Want to expose a simple, non-developer-friendly admin UI with zero code
* Want a basic admin panel up and running as a placeholder for a more advanced, custom-written one later

Not good for people who...

* Prefer code-level validations
* Want advanced functionality, because this be basic


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
