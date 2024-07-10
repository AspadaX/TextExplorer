// @generated automatically by Diesel CLI.

diesel::table! {
    configurations (id) {
        id -> Nullable<Integer>,
        service -> Text,
        version -> Text,
        data -> Text,
    }
}

diesel::table! {
    documents (id) {
        id -> Text,
        title -> Text,
        full_text -> Text,
        created_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    configurations,
    documents,
);
