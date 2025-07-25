use actix_web::HttpResponse;

#[derive(Clone)]
pub struct Route {
    pub path: String,
    pub method: Method,
}

#[derive(Clone)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
}

pub trait RouterHandler {
    fn handle(&self) -> HttpResponse;
}