// @generated automatically by Diesel CLI.

diesel::table! {
    matches (user1, user2) {
        user1 -> Text,
        user2 -> Text,
        state -> Integer,
    }
}

diesel::table! {
    messages (id) {
        id -> Integer,
        user1 -> Text,
        user2 -> Text,
        sender -> Integer,
        time -> Text,
        content -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Text,
        latitude -> Nullable<Float>,
        longitude -> Nullable<Float>,
        birth_date -> Nullable<Text>,
        name -> Nullable<Text>,
        gender_identity -> Nullable<Text>,
        pronouns -> Nullable<Text>,
        bio -> Nullable<Text>,
        looking_for -> Nullable<Text>,
        interests -> Nullable<Text>,
        photos -> Nullable<Text>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    matches,
    messages,
    users,
);
