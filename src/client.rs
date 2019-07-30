//! # client
//!
//! Implements the redis client capabilities.
//!

#[cfg(test)]
#[path = "./client_test.rs"]
mod client_test;

use crate::connection;
use redis;
use std::str::FromStr;
use crate::types::{
    ErrorInfo, RedisBoolResult, RedisEmptyResult, RedisError, RedisResult, RedisStringResult,
};

/// The redis client which enables to invoke redis operations.
pub struct Client {
    /// Internal redis client
    client: redis::Client,
    /// Holds the current redis connection
    connection: connection::Connection,
}

fn run_command_on_connection<T: redis::FromRedisValue>(
    connection: &mut redis::Connection,
    command: &str,
    args: Vec<&str>,
) -> RedisResult<T> {
    let mut cmd = redis::cmd(command);

    for arg in args {
        cmd.arg(arg);
    }

    let result: redis::RedisResult<T> = cmd.query(connection);

    match result {
        Err(error) => Err(RedisError {
            info: ErrorInfo::RedisError(error),
        }),
        Ok(output) => Ok(output),
    }
}

impl Client {
    /// Returns true if the currently stored connection is valid, otherwise false.<br>
    /// There is no need to call this function as any redis operation invocation will
    /// ensure a valid connection is created.
    pub fn is_connection_open(self: &mut Client) -> bool {
        self.connection.is_connection_open()
    }

    /// Closes the internal connection to redis.<br>
    /// The client can still be reused and any invocation of other operations after this call,
    /// will reopen the connection.<br>
    /// See redis [QUIT](https://redis.io/commands/quit) command.
    ///
    /// # Example
    ///
    /// ```
    /// # let mut client = simple_redis::create("redis://127.0.0.1:6379/").unwrap();
    /// match client.quit() {
    ///     Err(error) => println!("Error: {}", error),
    ///     _ => println!("Connection Closed.")
    /// }
    /// ```
    ///
    pub fn quit(self: &mut Client) -> RedisEmptyResult {
        let mut result;
        if self.is_connection_open() {
            result = self.run_command_empty_response("QUIT", vec![]);
        } else {
            result = Ok(());
        }

        result
    }

    /// Invokes the requested command with the provided arguments (all provided via args) and returns the operation
    /// response.<br>
    /// This function ensures that we have a valid connection and it is used internally by all other exposed
    /// commands.<br>
    /// This function is also public to enable invoking operations that are not directly exposed by the client.
    ///
    /// # Arguments
    ///
    /// * `command` - The Redis command, for example: `GET`
    /// * `args` - Vector of arguments for the given command
    ///
    /// # Example
    ///
    /// ```
    /// # let mut client = simple_redis::create("redis://127.0.0.1:6379/").unwrap();
    /// match client.run_command::<String>("ECHO", vec!["testing"]) {
    ///     Ok(value) => assert_eq!(value, "testing"),
    ///     _ => panic!("test error"),
    /// }
    /// ```
    pub fn run_command<T: redis::FromRedisValue>(
        self: &mut Client,
        command: &str,
        args: Vec<&str>,
    ) -> RedisResult<T> {
        match self.connection.get_redis_connection(&self.client) {
            Ok(ref mut connection) => run_command_on_connection::<T>(connection, command, args),
            Err(error) => Err(error),
        }
    }

    /// invokes the run_command and returns typed result
    pub fn run_command_from_string_response<T: FromStr>(
        self: &mut Client,
        command: &str,
        args: Vec<&str>,
    ) -> RedisResult<T> {
        match self.run_command::<String>(command, args) {
            Ok(value) => match T::from_str(&value) {
                Ok(typed_value) => Ok(typed_value),
                _ => Err(RedisError {
                    info: ErrorInfo::Description("Unable to parse output value."),
                }),
            },
            Err(error) => Err(error),
        }
    }

    /// invokes the run_command but returns empty result
    pub fn run_command_empty_response(
        self: &mut Client,
        command: &str,
        args: Vec<&str>,
    ) -> RedisEmptyResult {
        self.run_command(command, args)
    }

    /// invokes the run_command but returns string result
    pub fn run_command_string_response(
        self: &mut Client,
        command: &str,
        args: Vec<&str>,
    ) -> RedisStringResult {
        self.run_command(command, args)
    }

    /// invokes the run_command but returns bool result
    pub fn run_command_bool_response(
        self: &mut Client,
        command: &str,
        args: Vec<&str>,
    ) -> RedisBoolResult {
        self.run_command(command, args)
    }

}

/// Constructs a new redis client.<br>
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
///     match simple_redis::create("redis://127.0.0.1:6379/") {
///         Ok(client) => println!("Created Redis Client"),
///         Err(error) => println!("Unable to create Redis client: {}", error)
///     }
/// }
/// ```
pub fn create(connection_string: &str) -> Result<Client, RedisError> {
    match redis::Client::open(connection_string) {
        Ok(redis_client) => {
            let redis_connection = connection::create();

            let client = Client {
                client: redis_client,
                connection: redis_connection,
            };

            Ok(client)
        }
        Err(error) => Err(RedisError {
            info: ErrorInfo::RedisError(error),
        }),
    }
}
