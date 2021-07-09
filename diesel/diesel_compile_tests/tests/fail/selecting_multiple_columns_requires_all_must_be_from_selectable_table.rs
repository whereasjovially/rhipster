extern crate diesel;

use diesel::*;

table! {
    users {
        id -> Integer,
        name -> VarChar,
    }
}

table! {
    posts {
        id -> Integer,
        title -> VarChar,
        user_id -> Integer,
    }
}

fn main() {
    let stuff = users::table.select((posts::id, posts::user_id));
    let stuff = users::table.select((posts::id, users::name));
}
