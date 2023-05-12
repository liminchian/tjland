use chrono::TimeZone;
use rocket::{
    form::FromFormField,
    serde::{Deserialize, Serialize},
    FromForm,
};

use crate::middleware::{AffectedRows, Record};

#[derive(Debug, Serialize, Deserialize)]
pub struct UtcDateTime(chrono::DateTime<chrono::Utc>);

impl<'r> FromFormField<'r> for UtcDateTime {
    fn from_value(field: rocket::form::ValueField<'r>) -> rocket::form::Result<'r, UtcDateTime> {
        Ok(UtcDateTime(
            chrono::Utc
                .datetime_from_str(field.value, "%Y-%m-%dT%H:%M")
                .expect("Unable parse field."),
        ))
    }
}

#[derive(Debug, Serialize, Deserialize, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct User {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[rocket::async_trait]
pub trait UserTable {
    async fn create_user(
        &self,
        name: String,
        email: String,
        password: String,
    ) -> Result<Record, crate::error::Error>;
    async fn delete_user(&self, id: String) -> Result<AffectedRows, crate::error::Error>;
    async fn update_user(&self, id: String, user: User) -> Result<User, crate::error::Error>;
    async fn search_user(&self, id: String) -> Result<User, crate::error::Error>;
}

#[derive(Debug, FromForm, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Booking {
    pub subject: String,
    pub completed: bool,
    pub notified: bool,
    pub user_id: String,
    pub booking_at: UtcDateTime,
}

#[rocket::async_trait]
pub trait BookingTable {
    async fn create_booking(
        &self,
        subject: String,
        booking_at: UtcDateTime,
        user_id: String,
    ) -> Result<Record, crate::error::Error>;
    async fn delete_booking(&self, id: String) -> Result<AffectedRows, crate::error::Error>;
    async fn update_booking(
        &self,
        id: String,
        booking: Booking,
    ) -> Result<AffectedRows, crate::error::Error>;
    async fn search_booking(&self, id: String) -> Result<Booking, crate::error::Error>;
    async fn search_all_bookings(&self) -> Result<Vec<Booking>, crate::error::Error>;
}
