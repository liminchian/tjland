use std::sync::Arc;

use chrono::{DateTime, Utc};
use rocket::{
    fairing::{Fairing, Info, Kind, Result},
    Build, Rocket,
};
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    sql::Thing,
    Surreal,
};

use crate::data::{Booking, BookingTable, CheckItem, User, UserTable};

// from: https://www.workfall.com/learning/blog/use-surrealdb-to-persist-data-with-rocket-rest-api/
pub struct DbMiddleware;

#[rocket::async_trait]
impl Fairing for DbMiddleware {
    fn info(&self) -> Info {
        Info {
            name: "Database Middleware",
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> Result {
        let figment = rocket.figment().clone();
        let db_config: DbConfig = figment.select("database").extract().unwrap();
        let db = DbInstance::new(db_config.namespace, db_config.database)
            .await
            .unwrap();

        db.init_table().await.unwrap();

        Ok(rocket.manage(db))
    }
}

#[derive(Deserialize)]
struct DbConfig {
    namespace: String,
    database: String,
}

pub struct DbInstance(Arc<Surreal<Client>>);

#[derive(Debug, Serialize, Deserialize)]
struct Record {
    id: Thing,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AffectedRows {
    pub rows_affected: usize,
}

impl DbInstance {
    pub async fn new(
        namespace: String,
        database: String,
    ) -> Result<DbInstance, crate::error::Error> {
        let db = Surreal::new::<Ws>("127.0.0.1:24131").await?;
        db.use_ns(&namespace).use_db(&database).await?;
        db.signin(Root {
            username: "root",
            password: "root",
        })
        .await?;

        Ok(DbInstance(Arc::new(db)))
    }

    async fn init_table(&self) -> Result<(), crate::error::Error> {
        self.0.query("DEFINE TABLE booking;").await?;
        self.0.query("DEFINE TABLE user;").await?;
        Ok(())
    }

    fn format_id<'a>(id: &'a str, tb: &'a str) -> &'a str {
        match id.strip_prefix(format!("{}:", tb).as_str()) {
            Some(i) => i,
            None => id,
        }
    }
}

#[rocket::async_trait]
impl UserTable for DbInstance {
    async fn create_user(
        &self,
        name: String,
        email: String,
        password: String,
    ) -> Result<String, crate::error::Error> {
        let record: Record = self
            .0
            .create("user")
            .content(User {
                name,
                email,
                password,
            })
            .await?;
        let id = record.id.id.to_string();

        Ok(id)
    }

    async fn delete_user(&self, id: String) -> Result<AffectedRows, crate::error::Error> {
        let uid = Self::format_id(&id, "user");
        let _: User = self.0.delete(("user", uid)).await?;
        Ok(AffectedRows { rows_affected: 1 })
    }

    async fn update_user(&self, id: String, user: User) -> Result<User, crate::error::Error> {
        let uid = Self::format_id(&id, "user");
        Ok(self.0.update(("user", uid)).content(user).await?)
    }

    async fn search_user(&self, id: String) -> Result<User, crate::error::Error> {
        let uid = Self::format_id(&id, "user");
        Ok(self.0.select(("user", uid)).await?)
    }
}

#[rocket::async_trait]
impl BookingTable for DbInstance {
    async fn create_booking(
        &self,
        content: String,
        booking_at: DateTime<Utc>,
        user_id: String,
    ) -> Result<String, crate::error::Error> {
        let record: Record = self
            .0
            .create("booking")
            .content(Booking::new(content, user_id, booking_at))
            .await?;
        let id = record.id.id.to_string();

        Ok(id)
    }

    async fn delete_booking(&self, id: String) -> Result<AffectedRows, crate::error::Error> {
        let bid = Self::format_id(&id, "booking");
        let deleted: Option<Booking> = self.0.delete(("booking", bid)).await?;
        match deleted {
            Some(booking) => {
                dbg!(booking);
                Ok(AffectedRows { rows_affected: 1 })
            }
            None => Ok(AffectedRows { rows_affected: 0 }),
        }
    }

    async fn update_booking(
        &self,
        booking_id: String,
        booking: Booking,
    ) -> Result<Booking, crate::error::Error> {
        let bid = Self::format_id(&booking_id, "booking");
        Ok(self.0.update(("booking", bid)).content(booking).await?)
    }

    async fn search_booking(&self, id: String) -> Result<Booking, crate::error::Error> {
        let bid = Self::format_id(&id, "booking");
        Ok(self.0.select(("booking", bid)).await?)
    }

    async fn search_all_bookings(&self) -> Result<Vec<Booking>, crate::error::Error> {
        Ok(self.0.select("booking").await?)
    }

    async fn partial_update_booking(
        &self,
        booking_id: String,
        modified: CheckItem,
    ) -> Result<Booking, crate::error::Error> {
        let bid = Self::format_id(&booking_id, "booking");
        Ok(self.0.update(("booking", bid)).merge(modified).await?)
    }
}
