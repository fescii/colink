// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "institution_type"))]
    pub struct InstitutionType;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::InstitutionType;

    institutions (id) {
        id -> Int4,
        #[max_length = 250]
        short_name -> Varchar,
        #[max_length = 500]
        name -> Varchar,
        #[max_length = 500]
        logo -> Nullable<Varchar>,
        contact -> Nullable<Json>,
        in_type -> Nullable<InstitutionType>,
        active -> Nullable<Bool>,
        #[max_length = 500]
        location -> Nullable<Varchar>,
        about -> Nullable<Text>,
        established -> Nullable<Date>,
        #[max_length = 500]
        picture -> Nullable<Varchar>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 250]
        username -> Varchar,
        #[max_length = 500]
        password -> Varchar,
        #[max_length = 250]
        email -> Varchar,
        #[max_length = 250]
        name -> Varchar,
        active -> Nullable<Bool>,
        bio -> Nullable<Text>,
        dob -> Nullable<Timestamptz>,
        #[max_length = 500]
        picture -> Nullable<Varchar>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    institutions,
    users,
);
