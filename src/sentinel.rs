//! # sentinel
//!
//! Defines the functions related to Sentinels.
//!

use crate::client;
use redis;
use client::Client;

/// Encapsulates the metadata and connections related to the Sentinels
pub struct Sentinel {
    /// Encapsulates the metadata and connections related to the Sentinels
    pub sentinel_addrs: Vec<String>,
    /// Encapsulates the metadata and connections related to the Sentinels
    pub master: String,
    /// Encapsulates the metadata and connections related to the Sentinels
    pub master_client: Client,
}

impl Sentinel {

    /// Encapsulates the metadata and connections related to the Sentinels
    pub fn new(sentinel_addrs: Vec<String>, master: String) -> Sentinel
    {
        let master_client = Sentinel::new_client(sentinel_addrs.clone(), master.clone()).unwrap();
        Sentinel {
            sentinel_addrs,
            master,
            master_client
        }
    }

    fn is_connection_open(client: &mut Client) -> bool {
        match client.run_command_empty_response("PING", vec![]) {
            Ok(_) => {
                return true
            },
            Err(e) => {
                println!("Error! {:?}", e);
            }
        };
        false
    }

    /// Encapsulates the metadata and connections related to the Sentinels
    fn new_client(sentinel_addrs: Vec<String>, master: String) -> Option<Client> {
        for sentinel_addr in &sentinel_addrs {
            let sentinel_client = redis::Client::open(sentinel_addr.as_str()).unwrap();
            let mut con = sentinel_client.get_connection().unwrap();

            let current_master_info = redis::cmd("SENTINEL")
                .arg("get-master-addr-by-name")
                .arg(master.as_str()).query(&mut con);

            match current_master_info {
                Ok(addr) => {
                    let addr: Vec<String> = addr;
                    let current_master_socket = format!("redis://{}:{}/", &addr[0], &addr[1]);
                    println!("Current master socket: {}", current_master_socket.as_str());
                    let mut client = client::create(current_master_socket.as_str());
                    match client  {
                        Ok(mut c) => {
                            if Sentinel::is_connection_open(&mut c) {
                                return Some(c)
                            }
                        },
                        Err(e) => {
                            println!("Error: {:?}", e);
                        }
                    }
                },
                Err(e) => {
                    println!("Failed to get current master from sentinel: {:?}", e);
                }
            };
        }
        None
    }

    /// Retrieves the master client as reported by the Sentinel(s).
    pub fn get_client(&mut self) -> Option<&Client>
    {
        if !Sentinel::is_connection_open(&mut self.master_client) {
            return match Sentinel::new_client(self.sentinel_addrs.clone(), self.master.clone()) {
                Some(client) => {
                    self.master_client = client;
                    Some(&self.master_client)
                },
                None => {
                    println!("No client!");
                    None
                }
            };
        }
        Some(&self.master_client)
    }
}
