#[macro_use]
extern crate rocket;

use std::io::ErrorKind;

use rocket::{
    figment::{
        providers::{Format, Toml},
        Figment,
    },
    Config,
    {serde::json::Json, State},
};

use crate::data::{Booking, BookingTable, CheckItem, User, UserTable};
use crate::middleware::{AffectedRows, DbInstance, DbMiddleware};
use cors::*;

mod cors;
mod data;
mod error;
mod middleware;
mod prelude;
#[cfg(test)]
mod tests;
mod utils;

#[get("/booking")]
async fn get_all_bookings(db: &State<DbInstance>) -> Result<Json<Vec<Booking>>, std::io::Error> {
    Ok(Json(db.search_all_bookings().await.map_err(|_| {
        std::io::Error::new(ErrorKind::Other, "Unable get all bookings.")
    })?))
}

#[post("/", data = "<booking>")]
async fn create_booking(
    booking: Json<Booking>,
    db: &State<DbInstance>,
) -> Result<Json<String>, std::io::Error> {
    let obj = booking.into_inner();
    Ok(Json(
        db.create_booking(obj.content, obj.booking_at, obj.user_id)
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

#[put("/<booking_id>", data = "<item>")]
async fn update_booking(
    booking_id: String,
    item: Json<Booking>,
    db: &State<DbInstance>,
) -> Result<Json<Booking>, std::io::Error> {
    Ok(Json(
        db.update_booking(booking_id, item.into_inner())
            .await
            .map_err(|_| std::io::Error::new(ErrorKind::Other, "Unable update whole booking."))?,
    ))
}

#[patch("/<booking_id>")]
async fn complete_booking(
    booking_id: String,
    db: &State<DbInstance>,
) -> Result<Json<Booking>, std::io::Error> {
    let mut item = CheckItem::default();
    item.completed = true;
    Ok(Json(
        db.partial_update_booking(booking_id, item)
            .await
            .map_err(|_| std::io::Error::new(ErrorKind::Other, "Unable complete booking."))?,
    ))
}

#[patch("/notify/<booking_id>")]
async fn notify(
    booking_id: String,
    db: &State<DbInstance>,
) -> Result<Json<Booking>, std::io::Error> {
    let mut item = CheckItem::default();
    item.notified = true;
    Ok(Json(
        db.partial_update_booking(booking_id, item)
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
) -> Result<Json<String>, std::io::Error> {
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

#[put("/<user_id>", data = "<user>")]
async fn update_user(
    user_id: String,
    user: Json<User>,
    db: &State<DbInstance>,
) -> Result<Json<User>, std::io::Error> {
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
                update_booking,
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
