# Manager

Manager intends to provide basic but incredibly convenient CRUD on existing database tables.


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
