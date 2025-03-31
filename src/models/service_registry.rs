use std::collections::HashMap;
use chrono::prelude::*;
use chrono::TimeDelta;

// Implementation of the internal service entry.
struct ServiceEntry {
    name: String,
    last_update: DateTime<Utc>,
    metadata: HashMap<String, String>,
}


// The public implementation of the service registry is just
// a simple HashMap where the key is the service name.
// Keep it simple, right?
pub struct ServiceRegistry {
    services: HashMap<String, ServiceEntry>
}


impl ServiceRegistry {
    // "Constructor"
    pub fn new() -> Self {
        ServiceRegistry {
            services: HashMap::new(),
        }
    }

    // Add a service to the registry.
    // The name is the most important piece, but it is possible to associate arbitrary
    // KVP metadata to the service record.
    pub fn add_service(&mut self, name: String, metadata: HashMap<String, String>) {
        self.services.insert(name.clone(), {
            ServiceEntry {
                name: name.clone(),
                last_update: Utc::now(),
                metadata: metadata,
            }
        });
    }

    // Dumps out the registry into a string.
    pub fn dump(&self) -> String {
        let mut vec = self.services.iter()
            .map(|(_k, v)| { 
                format!("{}\n\t{}", v.name, 
                    v.metadata.iter()
                        .map(|(k, v)| format!("{}: {}", k, v))
                        .collect::<Vec<_>>()
                        .join("\n\t"))
                })
            .collect::<Vec<_>>();

        vec.sort();
        vec.join("\n")
    }

    // This is a simple cleanup function that removes any services that have not been updated in the last
    // period of time specified.
    pub fn cleanup(&mut self, expiry_time: TimeDelta) {
        let now = Utc::now();
        self.services.retain(|_, entry| {
            // Keep the service if it was updated within the last 5 minutes
            (now - entry.last_update) <= expiry_time
        });
    } 
}