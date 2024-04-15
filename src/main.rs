use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use futures::stream::TryStreamExt;
use mongodb::{
    bson::{self, oid::ObjectId},
    error::Error,
    Client, Collection,
};
use serde::{Deserialize, Serialize};

const DB_NAME: &str = "diary-notebook";
const COLL_NAME: &str = "calendars";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Calendar {
    #[serde(
        rename(serialize = "id", deserialize = "_id"),
        serialize_with = "bson::serde_helpers::serialize_object_id_as_hex_string"
    )]
    id: ObjectId,
    name: String,
    color: String,
    border_color: String,
    background_color: String,
    drag_background_color: String,
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[get("/api/v1/calendars")]
async fn get_calendars(client: web::Data<Client>) -> impl Responder {
    let collection: Collection<Calendar> = client.database(DB_NAME).collection(COLL_NAME);
    let cursor = match collection.find(None, None).await {
        Ok(cursor) => cursor,
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    };

    let calendars: Result<Vec<Calendar>, Error> = cursor.try_collect().await;

    return match calendars {
        Ok(calendars) => HttpResponse::Ok().json(calendars),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    };
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client = Client::with_uri_str("mongodb://localhost:27017")
        .await
        .expect("failed to connect");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .service(hello)
            .service(echo)
            .service(get_calendars)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
