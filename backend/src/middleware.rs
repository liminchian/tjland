// from: https://www.workfall.com/learning/blog/use-surrealdb-to-persist-data-with-rocket-rest-api/
use std::sync::Arc;

use rocket::{
    fairing::{Fairing, Info, Kind, Result},
    Build, Rocket,
};
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    sql::Thing,
    Surreal,
};

use crate::data::{Booking, BookingTable, User, UserTable, UtcDateTime};

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

        Ok(rocket.manage(db))
    }
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct DbConfig {
    namespace: String,
    database: String,
}

pub struct DbInstance(Arc<Surreal<Client>>);

impl DbInstance {
    pub async fn new(
        namespace: String,
        database: String,
    ) -> Result<DbInstance, crate::error::Error> {
        let db = Surreal::new::<Ws>("127.0.0.1:24131").await?;
        db.use_ns(&namespace).use_db(&database).await?;
        Ok(DbInstance(Arc::new(db)))
    }
}

#[rocket::async_trait]
impl UserTable for DbInstance {
    async fn create_user(
        &self,
        name: String,
        email: String,
        password: String,
    ) -> Result<Record, crate::error::Error> {
        Ok(self
            .0
            .create("user")
            .content(User {
                name,
                email,
                password,
            })
            .await?)
    }

    async fn delete_user(&self, id: String) -> Result<AffectedRows, crate::error::Error> {
        let _: Record = self.0.delete(("user", id.as_str())).await?;
        let deleted: Vec<String> = self
            .0
            .query("DELETE booking WHERE user_id = $id")
            .bind(id)
            .await?
            .take("id")?;

        Ok(AffectedRows {
            rows_affected: deleted.len() as u64,
        })
    }

    async fn update_user(
        &self,
        id: String,
        user: User,
    ) -> Result<AffectedRows, crate::error::Error> {
        let _: Record = self.0.update(("user", id.as_str())).content(user).await?;
        Ok(AffectedRows { rows_affected: 1 })
    }

    async fn search_user(&self, id: String) -> Result<User, crate::error::Error> {
        Ok(self.0.select(("user", id.as_str())).await?)
    }
}

#[rocket::async_trait]
impl BookingTable for DbInstance {
    async fn create_booking(
        &self,
        subject: String,
        booking_at: UtcDateTime,
        user_id: String,
    ) -> Result<Record, crate::error::Error> {
        Ok(self
            .0
            .create("booking")
            .content(Booking {
                subject,
                booking_at,
                user_id,
                completed: false,
                notified: false,
            })
            .await?)
    }

    async fn delete_booking(&self, id: String) -> Result<AffectedRows, crate::error::Error> {
        let deleted: Booking = self.0.delete(("booking", id.as_str())).await?;
        dbg!(deleted);
        Ok(AffectedRows { rows_affected: 1 })
    }

    async fn update_booking(
        &self,
        id: String,
        booking: Booking,
    ) -> Result<AffectedRows, crate::error::Error> {
        let updated: Booking = self
            .0
            .update(("booking", id.as_str()))
            .content(booking)
            .await?;
        dbg!(updated);
        Ok(AffectedRows { rows_affected: 1 })
    }

    async fn search_booking(&self, id: String) -> Result<Booking, crate::error::Error> {
        Ok(self.0.select(("booking", id.as_str())).await?)
    }

    async fn search_all_bookings(&self) -> Result<Vec<Booking>, crate::error::Error> {
        let result: Vec<Booking> = self.0.select("booking").await?;
        Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Record {
    #[allow(dead_code)]
    pub id: Thing,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AffectedRows {
    pub rows_affected: u64,
}
