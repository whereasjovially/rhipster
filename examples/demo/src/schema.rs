// @generated automatically by Diesel CLI.

diesel::table! {
    patients (id) {
        id -> Int4,
        first -> Varchar,
        last -> Varchar,
        dob -> Date,
    }
}

diesel::table! {
    practitioners (id) {
        id -> Int4,
        first -> Varchar,
        last -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    patients,
    practitioners,
);
