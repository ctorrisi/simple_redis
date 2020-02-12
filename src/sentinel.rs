//! # sentinel
//!
//! Defines the functions related to Sentinels.
//!

use crate::client;
use crate::types::RedisError;
use redis;
use client::Client;

/// Encapsulates the metadata and connections related to the Sentinels
pub struct Sentinel {
    sentinel_addrs: Vec<String>,
    master: String,
    client: Client,
}

impl Sentinel {

    /// Retrieves the master client as reported by the Sentinel(s).
    pub fn get_client(&mut self) -> Result<&Client, RedisError>
    {
        if !self.client.is_connection_open() {
            for sentinel_addr in &self.sentinel_addrs {
                let sentinel_client = redis::Client::open(sentinel_addr.as_str()).unwrap();
                let mut con = sentinel_client.get_connection().unwrap();

                let current_master_info = redis::cmd("SENTINEL")
                    .arg("get-master-addr-by-name")
                    .arg(self.master.as_str()).query(&mut con);

                let client: Option<Client> = match current_master_info {
                    Ok(addr) => {
                        let addr: Vec<String> = addr;
                        let current_master_socket = format!("{}:{}", &addr[0], &addr[1]);
                        println!("Current master socket: {}", current_master_socket.as_str());
                        let mut client = client::create(current_master_socket.as_str())?;
                        if client.is_connection_open() {
                            Some(client)
                        } else {
                            None
                        }
                    },
                    Err(e) => {
                        println!("Failed to get current master from sentinel: {:?}", e);
                        None
                    }
                };

                if client.is_some() {
                    self.client = client.unwrap();
                    break;
                }
            }
        }

        Ok(&self.client)
    }
}
