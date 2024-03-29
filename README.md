# Manager [PROTOTYPE]

Manager **intends** to provide basic but incredibly convenient CRUD on existing database tables,
where all you supply is the database URL and the UI (including forms and validations) is generated entirely from the database schema.

Good for people who...

* Prefer constraints and validations (and even object descriptions) to live in the database
* Want to expose a simple, non-developer-friendly admin UI with zero code
* Want a basic admin panel up and running as a placeholder for a more advanced, custom-written one later

Not good for people who...

* Prefer code-level validations
* Want advanced functionality, because this be basic

**IMPORTANT:** This is very alpha state and development is sporadic (1) because life, and (2) because
I'm still exploring good strategies for presenting database-level constraints (and validation failures)
to the user in a _clear enough_ format that is actually usable for non-DBAs.


## Running

While developing, use `cargo watch` to automatically re-check and re-build on file changes,
clearing the terminal screen on each reload:

```sh
$ cargo install cargo-watch

# To run admin app
$ DATABASE_URL="..." STATIC_PATH="static/" cargo watch --clear --exec 'run'
```
