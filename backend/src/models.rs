use diesel::prelude::*;
use chrono::NaiveDateTime;


#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::documents)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Document {
    pub id: String,
    pub title: String, 
    pub full_text: String, 
    pub created_at: NaiveDateTime
}