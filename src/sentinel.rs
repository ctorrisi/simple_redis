//! # sentinel
//!
//! Defines the functions related to Sentinels.
//!

use crate::client;
use client::Client;

/// Encapsulates the metadata and connections related to the Sentinels
pub struct Sentinel {
    /// Encapsulates the metadata and connections related to the Sentinels
    pub sentinel_addrs: Vec<String>,
    /// Encapsulates the metadata and connections related to the Sentinels
    pub master_name: String,
    /// Encapsulates the metadata and connections related to the Sentinels
    pub master_client: Client,
}

impl Sentinel {

    /// Encapsulates the metadata and connections related to the Sentinels
    pub fn new(sentinel_addrs: Vec<String>, master_name: String) -> Sentinel
    {
        let master_client = Sentinel::new_client(sentinel_addrs.clone(), master_name.clone()).unwrap();
        Sentinel {
            sentinel_addrs,
            master_name,
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

    fn get_master_addr(sentinel_addr: &str,  master: &str) -> String
    {
        let mut sentinel_client = client::create(sentinel_addr).unwrap();
        let args = vec!["get-master-addr-by-name", master];
        match sentinel_client.run_command_string_response("SENTINEL", args) {
            Ok(response) => {
                let full_addr: Vec<& str> = response.split_whitespace().collect();
                let master_addr = format!("redis://{}:{}/", full_addr[0], full_addr[1]);
                println!("master addr!: {}", master_addr);
                master_addr
            },
            Err(e) => {
                println!("Failed to get current master from sentinel {:?}: {:?}", sentinel_addr, e);
                String::new()
            }
        }
    }

    /// Encapsulates the metadata and connections related to the Sentinels
    fn new_client(sentinel_addrs: Vec<String>, master: String) -> Option<Client>
    {
        for sentinel_addr in &sentinel_addrs {
            let master_addr = Sentinel::get_master_addr(sentinel_addr.as_str(), &master.as_str());

            if !master_addr.is_empty() {
                let mut client = client::create(master_addr.as_str());
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
            };
        }
        None
    }

    /// Retrieves the master client as reported by the Sentinel(s).
    pub fn get_client(&mut self) -> Option<&mut Client>
    {
        if !Sentinel::is_connection_open(&mut self.master_client) {
            return match Sentinel::new_client(self.sentinel_addrs.clone(), self.master_name.clone()) {
                Some(client) => {
                    self.master_client = client;
                    Some(&mut self.master_client)
                },
                None => {
                    println!("No client!");
                    None
                }
            };
        }
        Some(&mut self.master_client)
    }
}
