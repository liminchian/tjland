use super::data::{Booking, User};
use super::middleware::AffectedRows;
use super::rocket;

use super::{
    rocket_uri_macro_cancel_booking, rocket_uri_macro_complete_booking,
    rocket_uri_macro_create_booking, rocket_uri_macro_create_user, rocket_uri_macro_delete_user,
    rocket_uri_macro_get_all_bookings, rocket_uri_macro_get_booking, rocket_uri_macro_get_user,
    rocket_uri_macro_notify, rocket_uri_macro_update_booking, rocket_uri_macro_update_user,
};
use async_once::AsyncOnce;
use chrono::Utc;
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
async fn test_user_lifecycle() {
    let client = CLIENT.get().await;
    let user = User {
        name: "test".to_string(),
        email: "abc@gmail.com".to_string(),
        password: "123".to_string(),
    };

    // create_user
    let created = client.post(uri!("/user", create_user())).json(&user);
    let mut response = created.dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    let user_id = response.into_string().await.unwrap().replace("\"", "");

    // get_user
    let searched = client.get(uri!("/user", get_user(&user_id.to_string())));
    response = searched.dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_json::<User>().await.unwrap(), user);

    // update_user
    let updated = client
        .put(uri!("/user", update_user(&user_id.to_string())))
        .json(&User {
            name: "Mitchell".to_string(),
            email: user.email.to_string(),
            password: user.password.to_string(),
        });
    response = updated.dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        &response.into_json::<User>().await.unwrap().name,
        "Mitchell"
    );

    // delete_user
    let deleted = client.delete(uri!("/user", delete_user(&user_id.to_string())));
    response = deleted.dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.into_json::<AffectedRows>().await.unwrap(),
        AffectedRows { rows_affected: 1 }
    )
}

#[rocket::async_test]
async fn test_booking_lifecycle() {
    let client = CLIENT.get().await;
    let mut now = Utc::now();
    let mut booking = Booking::new("abc".to_string(), "test".to_string(), now);

    // create_booking
    let created = client
        .post(uri!("/booking", create_booking()))
        .json(&booking);

    let mut response = created.dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    let booking_id = response.into_string().await.unwrap().replace("\"", "");

    // get_booking
    let searched = client.get(uri!("/booking", get_booking(&booking_id.to_string())));
    response = searched.dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_json::<Booking>().await.unwrap(), booking);

    // get_all_bookings
    let searched_all = client.get(uri!("/booking", get_all_bookings()));
    response = searched_all.dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    assert!(!response
        .into_json::<Vec<Booking>>()
        .await
        .unwrap()
        .is_empty());

    // update_booking
    now = Utc::now();
    booking = Booking {
        content: "new".to_string(),
        booking_at: now,
        user_id: "456".to_string(),
        notified: false,
        completed: false,
    };
    let updated = client
        .put(uri!("/booking", update_booking(&booking_id.to_string())))
        .json(&booking);
    response = updated.dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_json::<Booking>().await.unwrap(), booking);

    // complete_booking
    let completed = client.patch(uri!("/booking", complete_booking(&booking_id.to_string())));
    response = completed.dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    assert!(response.into_json::<Booking>().await.unwrap().completed);

    // notify
    let notified = client.patch(uri!("/booking", notify(&booking_id.to_string())));
    response = notified.dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    assert!(response.into_json::<Booking>().await.unwrap().notified);

    // cancel_booking
    let canceled = client.delete(uri!("/booking", cancel_booking(&booking_id.to_string())));
    response = canceled.dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.into_json::<AffectedRows>().await.unwrap(),
        AffectedRows { rows_affected: 1 }
    );
}
