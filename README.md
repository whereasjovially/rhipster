rhipster
---

> Because Java was so 10 years ago.


A quick tool that generates CRUD, JHipster compatible microservices from model definitions, in Rust, for Rust.


# Usage
Create diesel `up.sql` and `down.sql` migration files, then from the `rhipster` directory, run:
`cargo run <target name> <database url> <up file> <down file>`

# Progress
 - [ ] Generate SQL from JDL
 - [X] Generate Rust template
 - [X] Dockerfiles
 - [X] Setup database
 - [ ] Auth with JWT
 - [X] Generate ORM schema from SQL (`diesel_cli`)
 - [X] Generate Rust models from schema (`diesel_cli_ext`)
 - [X] Generate Rocket routes from schema (`sourcegen`)
 - [ ] Support for more complex relations
 - [ ] Generated protos (`diesel_cli_ext`)
