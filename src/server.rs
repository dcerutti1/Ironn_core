use actix_web::{test, web, App, HttpResponse, HttpServer, Result as ActixResult};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

#[derive(Clone)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

// Simple handler type - just async functions that return HttpResponse
type Handler =
    Arc<dyn Fn() -> Pin<Box<dyn Future<Output = ActixResult<HttpResponse>> + Send>> + Send + Sync>;

// Simple route definition
#[derive(Clone)]
pub struct PubRoute {
    pub path: String,
    pub method: HttpMethod,
    pub handler: Handler,
}

//MAIN API:
pub struct IronnServer {
    routes: Vec<PubRoute>,
}

impl IronnServer {
    /// Create a new IronnServer instance
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    /// Add a public route to the server
    pub fn public_route<F, Fut>(mut self, path: &str, method: HttpMethod, handler: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ActixResult<HttpResponse>> + Send + 'static,
    {
        let handler: Handler = Arc::new(move || Box::pin(handler()));
        self.routes.push(PubRoute {
            path: path.to_string(),
            handler,
            method,
        });
        self
    }

    /// Start the HTTP server on default port 8080
    pub async fn run(self) -> Result<(), std::io::Error> {
        self.bind("127.0.0.1:8080").await
    }

    /// Start the HTTP server on a custom address
    pub async fn bind(self, address: &str) -> Result<(), std::io::Error> {
        let routes = Arc::new(self.routes);

        println!("🚀 IronnServer starting on http://{}", address);
        println!("📋 Routes registered: {}", routes.len());

        HttpServer::new(move || {
            let routes = Arc::clone(&routes);
            let mut app = App::new();

            for route in routes.iter() {
                let handler = Arc::clone(&route.handler);
                println!("📍 Registering: GET {}", route.path);

                match &route.method {
                    HttpMethod::Get => {
                        app = app.route(
                            &route.path,
                            web::get().to(move || {
                                let handler = Arc::clone(&handler);
                                async move { handler().await }
                            }),
                        );
                    }
                    HttpMethod::Post => {
                        app = app.route(
                            &route.path,
                            web::post().to(move || {
                                let handler = Arc::clone(&handler);
                                async move { handler().await }
                            }),
                        );
                    }
                    HttpMethod::Put => {
                        app = app.route(
                            &route.path,
                            web::put().to(move || {
                                let handler = Arc::clone(&handler);
                                async move { handler().await }
                            }),
                        );
                    }
                    HttpMethod::Delete => {
                        app = app.route(
                            &route.path,
                            web::delete().to(move || {
                                let handler = Arc::clone(&handler);
                                async move { handler().await }
                            }),
                        );
                    }
                }
            }
            app
        })
        .bind(address)?
        .run()
        .await
    }

    /// Get the number of registered routes
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }

    /// Create an App for testing (internal use)
    pub fn create_app(
        self,
    ) -> App<impl actix_web::dev::ServiceFactory<actix_web::dev::ServiceRequest>> {
        let mut app = App::new();
        for route in self.routes.iter() {
            let handler = Arc::clone(&route.handler);
            match &route.method {
                HttpMethod::Get => {
                    app = app.route(
                        &route.path,
                        web::get().to(move || {
                            let handler = Arc::clone(&handler);
                            async move { handler().await }
                        }),
                    );
                }
                HttpMethod::Post => {
                    app = app.route(
                        &route.path,
                        web::post().to(move || {
                            let handler = Arc::clone(&handler);
                            async move { handler().await }
                        }),
                    );
                }
                HttpMethod::Put => {
                    app = app.route(
                        &route.path,
                        web::put().to(move || {
                            let handler = Arc::clone(&handler);
                            async move { handler().await }
                        }),
                    );
                }
                HttpMethod::Delete => {
                    app = app.route(
                        &route.path,
                        web::delete().to(move || {
                            let handler = Arc::clone(&handler);
                            async move { handler().await }
                        }),
                    );
                }
            }
        }
        app
    }
}

// 🧪 TEST UTILITIES: Separate functions for easy testing

/// Test utility: Create a simple text response
pub fn text_response(
    content: &'static str,
) -> impl Fn() -> Pin<Box<dyn Future<Output = ActixResult<HttpResponse>> + Send>> + Send + Sync + 'static
{
    move || Box::pin(async move { Ok(HttpResponse::Ok().body(content)) })
}

/// Test utility: Create a JSON response
pub fn json_response<T>(
    data: T,
) -> impl Fn() -> Pin<Box<dyn Future<Output = ActixResult<HttpResponse>> + Send>> + Send + Sync + 'static
where
    T: serde::Serialize + Send + Sync + 'static + Clone,
{
    move || {
        let data = data.clone();
        Box::pin(async move { Ok(HttpResponse::Ok().json(data)) })
    }
}

/// Test utility: Create a health check response
pub fn health_check(
) -> impl Fn() -> Pin<Box<dyn Future<Output = ActixResult<HttpResponse>> + Send>> + Send + Sync + 'static
{
    || {
        Box::pin(async {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "status": "healthy",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        })
    }
}

/// Test utility: Create an error response
pub fn error_response(
    message: &'static str,
) -> impl Fn() -> Pin<Box<dyn Future<Output = ActixResult<HttpResponse>> + Send>> + Send + Sync + 'static
{
    move || Box::pin(async move { Err(actix_web::error::ErrorInternalServerError(message)) })
}

/// Test utility: Make a GET request to a test service
#[cfg(test)]
pub async fn get_request<S>(srv: &S, path: &str) -> actix_web::dev::ServiceResponse
where
    S: actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
{
    use actix_web::test;
    let req = test::TestRequest::get().uri(path).to_srv_request();
    test::call_service(srv, req).await
}

/// Test utility: Check if response status matches expected
#[cfg(test)]
pub async fn assert_status<S>(srv: &S, path: &str, expected_status: u16)
where
    S: actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
{
    let resp = get_request(srv, path).await;
    assert_eq!(
        resp.status().as_u16(),
        expected_status,
        "Wrong status for {}",
        path
    );
}

/// Test utility: Get response body as string
#[cfg(test)]
pub async fn get_body_string<S>(srv: &S, path: &str) -> String
where
    S: actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
{
    use actix_web::test;
    let resp = get_request(srv, path).await;
    let body = test::read_body(resp).await;
    String::from_utf8(body.to_vec()).unwrap()
}

/// Test utility: Get response body as JSON
#[cfg(test)]
pub async fn get_body_json<S>(srv: &S, path: &str) -> serde_json::Value
where
    S: actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
{
    use actix_web::test;
    let resp = get_request(srv, path).await;
    let body = test::read_body(resp).await;
    serde_json::from_slice(&body).unwrap()
}

// 🎯 EXAMPLE HANDLERS: Ready-to-use handlers for common cases

/// Example handler: Simple home page
pub async fn home_handler() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().body("Welcome to IronnServer! 🚀"))
}

/// Example handler: API status
pub async fn status_handler() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "running",
        "version": "1.0.0"
    })))
}

/// Example handler: Users list
pub async fn users_handler() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(vec![
        serde_json::json!({"id": 1, "name": "Alice"}),
        serde_json::json!({"id": 2, "name": "Bob"}),
    ]))
}

pub fn test() {
    println!("test");
}
pub fn test2() {
    println!("test 2 - ignore");
}
pub fn test3() {
    println!("test3")
}
