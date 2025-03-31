mod models;

use actix_web::{web, App, HttpServer, HttpResponse};
use chrono::TimeDelta;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use std::collections::HashMap;
use std::io::Result;
use std::sync::Arc;

use models::ServiceRegistry;



// Implementation of AppState. Could be in the models folder, but ehh...
// there's very little going on here right now.
struct AppState {
    registry: Arc<Mutex<ServiceRegistry>>,
}


// This function is run in the background and periodically cleans up the service registry.
// TBD: incorporate a configuration file.
async fn clear_service(registry: Arc<Mutex<ServiceRegistry>>) {
    loop {
        {
            let mut reg = registry.lock().await;
            reg.cleanup(TimeDelta::minutes(5));
        }

        sleep(Duration::from_secs(5)).await;
    }
}


// Register endoint for services to... well, register.
//
// The data parameter contains the global app state and the path currently contains the one true parameter,
// the name of the service. However, in the future, this may change to (String, String) to contain a GUID.
//
// The query parameter contains the metadata for the service. This contains arbitrary data that can be advertised
// to the user. These parameters will be publicly advertised via the services listing endpoint to display random
// statistics, like uptime, CPU usage, etc.
#[actix_web::get("/register/{name}")]
async fn add_service(data: web::Data<AppState>, query: web::Query<HashMap<String, String>>, path: web::Path<String>) -> HttpResponse {
    let mut registry = data.registry.lock().await;

    registry.add_service(path.to_string(), query.into_inner());
    HttpResponse::Ok().body("Registered Successfully")
}


// This is an endpoint to dump out the current list of services to the client.
// The registry itself implements a to_string version to dump out its current
// state, but in the future I plan to add a query parameter to allow for filtering
// only for what you want instead of needing to do it client-side.
#[actix_web::get("/services")]
async fn get_services(data: web::Data<AppState>) -> HttpResponse {
    let registry = data.registry.lock().await;
    let services = registry.dump();
    HttpResponse::Ok().body(services)
}


#[tokio::main]
async fn main() -> Result<()>{
    // Initialize our service registry
    let registry = Arc::new(Mutex::new(ServiceRegistry::new()));

    // Register a background process to periodically clean up older services.
    // Once registered, a service can renew by sending another API request to the register function.
    let cleanup_instance = Arc::clone(&registry);
    tokio::spawn(async move {
        clear_service(cleanup_instance).await;
    });

    // Start the web server.
    HttpServer::new(move || {
        // Create an app state for our webserver that will contain a reference
        // to our service registry. And any other things we might need later like
        // a config file.
        let app_state = AppState {
            registry: Arc::clone(&registry),
        };

        App::new()
            .app_data(web::Data::new(app_state))
            .service(get_services) // Note for future reference, use .service instead of .route when using actix_web::get/post macros.
            .service(add_service) 
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
