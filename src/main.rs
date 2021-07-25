use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};

async fn greet(req: HttpRequest) -> impl Responder {                //impl Responder is using the impl Trait syntax 
    let name = req.match_info().get("name").unwrap_or("World");     //return "something" that implements the Responder trait
    format!("Hello {}!", &name)
}

async fn health_check() -> impl Responder {                         //my first endpoint
    HttpResponse::Ok()
}

#[actix_web::main]                                                  //is to give us the illusion of being able to define an
//                                                                  //  asynchronous main while, under the hood, it just takes our main asynchronous body and writes the
//                                                                  //  necessary boilerplate to make it run on top of actix’s runtime.

async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()                                                  //gives us a clean slate to which we can add, one bit at a time, new behaviour using a fluent API
            .route("/", web::get().to(greet))                       //is a short-cut for Route::new().guard(guard::Get())
            .route("/{name}", web::get().to(greet))
            .route("/health_check", web::get().to(health_check))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
