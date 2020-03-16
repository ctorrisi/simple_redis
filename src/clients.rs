//! # client
//!
//! Implements the redis client capabilities.
//!

use redis::{Client, Connection, RedisResult};
use crate::types::{ErrorInfo, RedisError};

/// The redis client which enables to invoke redis operations.
pub struct Clients {
    clients: Vec<Client>,
    next_idx: usize
}

impl Clients {
    /// Returns an active client
    pub fn get_connection(&mut self) -> Result<Connection, RedisError> {
        let num_clients = self.clients.len();
        let mut idx = self.next_idx;
        let mut connection = None;
        let mut attempts = 0;

        while connection.is_none() && attempts < num_clients
        {
            let c = &self.clients[idx];
            match c.get_connection() {
                Ok(mut conn) => {
                    let res: RedisResult<()> = redis::cmd("PING").query(&mut conn);
                    connection = match res {
                        Ok(_) => Some(conn),
                        Err(e) => {
                            println!("Redis connection error! {:?}", e);
                            None
                       }
                    };
                },
                Err(e) => println!("Redis connection error! {:?}", e)
            }
            if connection.is_some() {
                break;
            }
            attempts = attempts + 1;
            idx = (idx + 1) % num_clients;
        }

        match connection {
            Some(conn) => Ok(conn),
            None => Err(RedisError { info: ErrorInfo::Description("Unable to connect to a client.") })
        }
    }
}

/// Constructs a collection of redis clients.<br>
/// The redis connection string must be in the following format: `redis://[:<passwd>@]<hostname>[:port][/<db>]`
///
/// # Arguments
///
/// * `connection_string` - The connection string in the format of: `redis://[:<passwd>@]<hostname>[:port][/<db>]`
///
/// # Example
///
/// ```
/// extern crate simple_redis;
/// fn main() {
///     match simple_redis::clients::create(vec!["redis://127.0.0.1:6379/", "redis://127.0.0.1:6380/"]) {
///         Ok(client) => println!("Created Redis Client"),
///         Err(error) => println!("Unable to create Redis client: {}", error)
///     }
/// }
/// ```
pub fn create(nodes: Vec<&str>) -> Result<Clients, RedisError> {
    let nodes_len = nodes.len();
    let mut clients: Vec<Client> = Vec::new();
    for node in nodes {
        match redis::Client::open(node) {
            Ok(c) => clients.push(c),
            Err(_) => break
        };
    }
    if clients.len() < nodes_len {
        Err(RedisError { info: ErrorInfo::Description("Unable to connect to all clients.") })
    } else {
        Ok(Clients { clients, next_idx: 0 })
    }
}
