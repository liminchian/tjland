#[macro_use]
extern crate rocket;

use std::io::ErrorKind;

use data::{BookingTable, UserTable};
use rocket::{
    figment::{
        providers::{Format, Toml},
        Figment,
    },
    Config,
    {serde::json::Json, State},
};

use crate::data::{Booking, User};
use crate::middleware::{AffectedRows, DbInstance, DbMiddleware, Record};
use cors::*;

mod cors;
mod data;
mod error;
mod middleware;
mod prelude;
mod utils;

#[get("/booking")]
async fn get_all_bookings(db: &State<DbInstance>) -> Result<Json<Vec<Booking>>, std::io::Error> {
    Ok(Json(db.search_all_bookings().await.map_err(|_| {
        std::io::Error::new(ErrorKind::Other, "Unable get all bookings.")
    })?))
}

#[post("/<user_id>", data = "<booking>")]
async fn create_booking(
    user_id: String,
    booking: Json<Booking>,
    db: &State<DbInstance>,
) -> Result<Json<Record>, std::io::Error> {
    let obj = booking.into_inner();
    Ok(Json(
        db.create_booking(obj.subject, obj.booking_at, user_id)
            .await
            .map_err(|_| std::io::Error::new(ErrorKind::Other, "Unable create booking."))?,
    ))
}

#[get("/<booking_id>")]
async fn get_booking(
    booking_id: String,
    db: &State<DbInstance>,
) -> Result<Json<Booking>, std::io::Error> {
    Ok(Json(db.search_booking(booking_id).await.map_err(|_| {
        std::io::Error::new(ErrorKind::Other, "Unable get booking.")
    })?))
}

#[patch("/<booking_id>")]
async fn complete_booking(
    booking_id: String,
    db: &State<DbInstance>,
) -> Result<Json<AffectedRows>, std::io::Error> {
    let booking = db.search_booking(booking_id.clone()).await.unwrap();
    Ok(Json(
        db.update_booking(
            booking_id,
            Booking {
                subject: booking.subject,
                completed: true,
                notified: false,
                user_id: booking.user_id,
                booking_at: booking.booking_at,
            },
        )
        .await
        .map_err(|_| std::io::Error::new(ErrorKind::Other, "Unable complete booking."))?,
    ))
}

#[patch("/notify/<booking_id>")]
async fn notify(
    booking_id: String,
    db: &State<DbInstance>,
) -> Result<Json<AffectedRows>, std::io::Error> {
    let booking = db.search_booking(booking_id.clone()).await.unwrap();
    Ok(Json(
        db.update_booking(
            booking_id,
            Booking {
                subject: booking.subject,
                completed: booking.completed,
                notified: true,
                user_id: booking.user_id,
                booking_at: booking.booking_at,
            },
        )
        .await
        .map_err(|_| std::io::Error::new(ErrorKind::Other, "Unable notify."))?,
    ))
}

#[delete("/<booking_id>")]
async fn cancel_booking(
    booking_id: String,
    db: &State<DbInstance>,
) -> Result<Json<AffectedRows>, std::io::Error> {
    Ok(Json(db.delete_booking(booking_id).await.map_err(|_| {
        std::io::Error::new(ErrorKind::Other, "Unable cancel booking.")
    })?))
}

#[post("/", data = "<user>")]
async fn create_user(
    user: Json<User>,
    db: &State<DbInstance>,
) -> Result<Json<Record>, std::io::Error> {
    let data = user.into_inner();
    Ok(Json(
        db.create_user(data.name, data.email, data.password)
            .await
            .map_err(|_| std::io::Error::new(ErrorKind::Other, "Unable create user."))?,
    ))
}

#[get("/<user_id>")]
async fn get_user(user_id: String, db: &State<DbInstance>) -> Result<Json<User>, std::io::Error> {
    Ok(Json(db.search_user(user_id).await.map_err(|_| {
        std::io::Error::new(ErrorKind::Other, "Unable get user.")
    })?))
}

#[post("/<user_id>", data = "<user>")]
async fn update_user(
    user_id: String,
    user: Json<User>,
    db: &State<DbInstance>,
) -> Result<Json<AffectedRows>, std::io::Error> {
    Ok(Json(
        db.update_user(user_id, user.into_inner())
            .await
            .map_err(|_| std::io::Error::new(ErrorKind::Other, "Unable update user."))?,
    ))
}

#[delete("/<user_id>")]
async fn delete_user(
    user_id: String,
    db: &State<DbInstance>,
) -> Result<Json<AffectedRows>, std::io::Error> {
    Ok(Json(db.delete_user(user_id).await.map_err(|_| {
        std::io::Error::new(ErrorKind::Other, "Unable delete user.")
    })?))
}

#[launch]
async fn rocket() -> _ {
    let figment = Figment::from(Config::default())
        .merge(Toml::file("Rocket.toml").nested())
        .merge(Toml::file("App.toml").nested());

    rocket::custom(figment)
        .mount(
            "/booking",
            routes![
                get_all_bookings,
                create_booking,
                get_booking,
                complete_booking,
                cancel_booking,
                notify,
            ],
        )
        .mount(
            "/user",
            routes![create_user, get_user, update_user, delete_user],
        )
        .attach(CORS)
        .attach(DbMiddleware)
}

#[cfg(test)]
mod test {
    use super::data::User;
    use super::rocket;
    use async_once::AsyncOnce;
    use lazy_static::lazy_static;
    use rocket::http::Status;
    use rocket::local::asynchronous::Client;

    lazy_static! {
        static ref CLIENT: AsyncOnce<Client> = AsyncOnce::new(async {
            Client::tracked(rocket().await)
                .await
                .expect("valid rocket instance")
        });
    }

    #[rocket::async_test]
    async fn test_create_user() {
        let user = User {
            name: "test".to_string(),
            email: "abc@gmail.com".to_string(),
            password: "123".to_string(),
        };
        let response = CLIENT
            .get()
            .await
            .post(uri!("/user/"))
            .json(&user)
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
    }

    #[rocket::async_test]
    async fn test_get_user() {
        let response = CLIENT
            .get()
            .await
            .get(uri!("/user/user:zv8cfwnz35972pq67dp9"))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
    }
}
