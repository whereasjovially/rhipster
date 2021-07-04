// @generated automatically by Diesel CLI.

diesel::table! {
    /// Representation of the `comments` table.
    ///
    /// (Automatically generated by Diesel.)
    comments (id) {
        /// The `id` column of the `comments` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        id -> Int4,
        /// The `post_id` column of the `comments` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        post_id -> Int4,
    }
}

diesel::table! {
    /// Representation of the `posts` table.
    ///
    /// (Automatically generated by Diesel.)
    posts (id) {
        /// The `id` column of the `posts` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        id -> Int4,
        /// The `user_id` column of the `posts` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        user_id -> Int4,
    }
}

diesel::table! {
    /// Representation of the `users` table.
    ///
    /// (Automatically generated by Diesel.)
    users (id) {
        /// The `id` column of the `users` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        id -> Int4,
    }
}

diesel::joinable!(comments -> posts (post_id));
diesel::joinable!(posts -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    comments,
    posts,
    users,
);
