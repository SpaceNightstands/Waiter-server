use super::prelude::*;

pub fn get_service() -> actix_web::Scope{
	web::scope("/auth")
    .route("", web::get().to(get_jwt))
}

/*TODO: Create and send back jwt
 *(or any other kind of authentication token)*/
async fn get_jwt() -> impl Responder {
	"Hello, world"
}

//TODO: Create Route Guard
