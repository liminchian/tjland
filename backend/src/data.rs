use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::middleware::AffectedRows;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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
    ) -> Result<String, crate::error::Error>;
    async fn delete_user(&self, id: String) -> Result<AffectedRows, crate::error::Error>;
    async fn update_user(&self, id: String, user: User) -> Result<User, crate::error::Error>;
    async fn search_user(&self, id: String) -> Result<User, crate::error::Error>;
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(crate = "rocket::serde")]
pub struct Booking {
    pub content: String,
    pub booking_at: DateTime<Utc>,
    pub user_id: String,
    pub completed: bool,
    pub notified: bool,
}

impl Booking {
    pub fn new(content: String, user_id: String, booking_at: DateTime<Utc>) -> Self {
        Booking {
            content,
            booking_at,
            user_id,
            completed: false,
            notified: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CheckItem {
    pub completed: bool,
    pub notified: bool,
}

#[rocket::async_trait]
pub trait BookingTable {
    async fn create_booking(
        &self,
        subject: String,
        booking_at: DateTime<Utc>,
        user_id: String,
    ) -> Result<String, crate::error::Error>;
    async fn delete_booking(&self, booking_id: String)
        -> Result<AffectedRows, crate::error::Error>;
    async fn update_booking(
        &self,
        id: String,
        booking: Booking,
    ) -> Result<Booking, crate::error::Error>;
    async fn partial_update_booking(
        &self,
        booking_id: String,
        item: CheckItem,
    ) -> Result<Booking, crate::error::Error>;
    async fn search_booking(&self, booking_id: String) -> Result<Booking, crate::error::Error>;
    async fn search_all_bookings(&self) -> Result<Vec<Booking>, crate::error::Error>;
}
