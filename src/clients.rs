//! # clients
//!
//! A wrapper for redis client, allowing connections to multiple clients.
//!

use std::time::Instant;
use redis::{cmd, Client, Connection, RedisResult, Commands};
use crate::types::{ErrorInfo, RedisError};

/// The redis client which enables to invoke redis operations.
pub struct Clients {
    clients: Vec<Client>,
    connection: Option<Connection>
}

impl Clients {
    fn check_connection(&mut self) {
        if self.connection.is_none() {
            self.reconnect();
        };
    }

    /// Invokes the SET command to redis
    pub fn set(&mut self, k: &str, v: &str) -> Result<(), RedisError> {
        self.check_connection();
        if let Some(mut conn) = self.connection.as_mut() {
            match conn.set(k, v) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    println!("Redis error on SET command: {:?}", e);
                    self.reconnect();
                    if let Some(mut c) = self.connection.as_mut() {
                        match c.set(k, v) {
                            Ok(()) => return Ok(()),
                            Err(e) => println!("Redis error on SET command: {:?}", e)
                        }
                    }
                }
            }
        }
        Err(RedisError { info: ErrorInfo::Description("Unable to connect to a client.") })
    }

    /// Invokes the DEL command to redis for an array of keys
    pub fn del_keys(&mut self, arg: &[String]) -> Result<(), RedisError> {
        self.check_connection();
        if let Some(mut conn) = self.connection.as_mut() {
            match cmd("DEL").arg(arg).query(conn) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    println!("Redis error on DEL command: {:?}", e);
                    self.reconnect();
                    if let Some(mut c) = self.connection.as_mut() {
                        match cmd("DEL").arg(arg).query(c) {
                            Ok(()) => return Ok(()),
                            Err(e) => println!("Redis error on DEL command: {:?}", e)
                        }
                    }
                }
            }
        }
        Err(RedisError { info: ErrorInfo::Description("Unable to connect to a client.") })
    }

    /// Invokes the DEL command to redis for a single key
    pub fn del(&mut self, k: &str) -> Result<(), RedisError> {
        self.check_connection();
        if let Some(mut conn) = self.connection.as_mut() {
            match conn.del(k) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    println!("Redis error on DEL command: {:?}", e);
                    self.reconnect();
                    if let Some(mut c) = self.connection.as_mut() {
                        match c.del(k) {
                            Ok(()) => return Ok(()),
                            Err(e) => println!("Redis error on DEL command: {:?}", e)
                        }
                    }
                }
            }
        }
        Err(RedisError { info: ErrorInfo::Description("Unable to connect to a client.") })
    }


    fn reconnect(&mut self) {
        let mut lowest_latency = std::u128::MAX;
        let mut connection = None;
        for c in &self.clients {
            match c.get_connection() {
                Ok(mut conn) => {
                    let start = Instant::now();
                    let res: RedisResult<()> = cmd("PING").query(&mut conn);
                    match res {
                        Ok(_) => {
                            let latency = start.elapsed().as_nanos();
                            if latency < lowest_latency {
                                lowest_latency = latency;
                                connection = Some(conn);
                                println!("Selected client: {:?}", c);
                            }
                        },
                        Err(e) => {
                            println!("Redis connection error! {:?}", e);
                        }
                    };
                },
                Err(e) => println!("Redis connection error! {:?}", e)
            }
        }
        self.connection = connection;
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
        Ok(Clients { clients, connection: None })
    }
}
