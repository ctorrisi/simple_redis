//! # sentinel
//!
//! Defines the functions related to Sentinels.
//!

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::client;
use client::Client;

/// Encapsulates the metadata and connections related to the Sentinels
pub struct Sentinel {
    /// Encapsulates the metadata and connections related to the Sentinels
    pub sentinel_addrs: Vec<String>,
    /// Encapsulates the metadata and connections related to the Sentinels
    pub master_name: String,
    /// Encapsulates the metadata and connections related to the Sentinels
    pub master_client: Arc<Mutex<Client>>,
    master_addr: Arc<Mutex<String>>
}

impl Sentinel {

    /// Encapsulates the metadata and connections related to the Sentinels
    pub fn new(sentinel_addrs: Vec<String>, master_name: String) -> Sentinel
    {
        let master_addr = Sentinel::get_master_addr(&sentinel_addrs, master_name.as_str());
        let master_client = Sentinel::create_new_client(&master_addr).unwrap();
        let mut sentinel = Sentinel {
            sentinel_addrs,
            master_name,
            master_client: Arc::new(Mutex::new(master_client)),
            master_addr: Arc::new(Mutex::new(master_addr))
        };
        sentinel.run_sentinel_monitor();
        sentinel
    }

    fn run_sentinel_monitor(&mut self)
    {
        let sentinel_addrs = self.sentinel_addrs.clone();
        let master_name = self.master_name.clone();
        let master_addr = self.master_addr.clone();
        let client = self.master_client.clone();
        thread::spawn(move || {
            loop {
                let addr = Sentinel::get_master_addr(&sentinel_addrs, master_name.as_str());
                if !addr.is_empty()  {
                    let mut current_addr = master_addr.lock().unwrap();
                    if addr.as_str() != current_addr.as_str() {
                        match Sentinel::create_new_client(&addr) {
                            Some(c) => {
                                let mut client_mut = client.lock().unwrap();
                                *client_mut = c;
                                *current_addr = addr;
                            },
                            None => {
                                println!("Failed to update client to address {}", addr.as_str());
                            }
                        }
                    }
                }
                thread::sleep(Duration::from_secs(30));
            }
        });
    }

    fn is_connection_open(client: &mut Client) -> bool
    {
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

    fn get_master_addr(sentinel_addrs: &Vec<String>,  master: &str) -> String
    {
        let mut master_addr = String::new();
        for sentinel_addr in sentinel_addrs {
            let mut sentinel_client = client::create(sentinel_addr).unwrap();
            match sentinel_client.run_command::<Vec<String>>("SENTINEL", vec!["get-master-addr-by-name", master]) {
                Ok(addr) => {
                    master_addr = format!("redis://{}:{}/", addr[0], addr[1]).to_string();
                    break;
                },
                Err(e) => {
                    println!("Failed to get current master from sentinel {:?}: {:?}", sentinel_addr, e);
                }
            }
        }
        master_addr
    }

    /// Encapsulates the metadata and connections related to the Sentinels
    fn create_new_client(master_addr: &String) -> Option<Client>
    {
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
        None
    }

    /// Retrieves the master client as reported by the Sentinel(s).
    pub fn get_client(&mut self) -> Option<Arc<Mutex<Client>>>
    {
        let mut client = self.master_client.lock().unwrap();
        if !Sentinel::is_connection_open(&mut client) {
            let master_addr = Sentinel::get_master_addr(&self.sentinel_addrs, self.master_name.as_str());
            return match Sentinel::create_new_client(&master_addr) {
                Some(c) => {
                    let mut master_addr_mut = self.master_addr.lock().unwrap();
                    *client = c;
                    *master_addr_mut = master_addr;
                    Some(self.master_client.clone())
                },
                None => {
                    None
                }
            }
        }
        Some(self.master_client.clone())
    }
}
