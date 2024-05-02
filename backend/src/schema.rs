// @generated automatically by Diesel CLI.

diesel::table! {
    impressions (user_id, profile_id) {
        user_id -> Text,
        profile_id -> Text,
        liked -> Integer,
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
    impressions,
    users,
);
