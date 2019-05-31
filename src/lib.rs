#![deny(
    absolute_paths_not_starting_with_crate,
    anonymous_parameters,
    keyword_idents,
    const_err,
    dead_code,
    deprecated,
    duplicate_macro_exports,
    ellipsis_inclusive_range_patterns,
    exceeding_bitshifts,
    illegal_floating_point_literal_pattern,
    improper_ctypes,
    incoherent_fundamental_impls,
    intra_doc_link_resolution_failure,
    invalid_type_param_default,
    irrefutable_let_patterns,
    late_bound_lifetime_arguments,
    legacy_constructor_visibility,
    legacy_directory_ownership,
    macro_use_extern_crate,
    missing_copy_implementations,
    missing_docs,
    missing_fragment_specifier,
    mutable_transmutes,
    no_mangle_const_items,
    no_mangle_generic_items,
    non_camel_case_types,
    non_shorthand_field_patterns,
    non_snake_case,
    non_upper_case_globals,
    overflowing_literals,
    parenthesized_params_in_types_and_modules,
    path_statements,
    patterns_in_fns_without_body,
    plugin_as_library,
    private_in_public,
    proc_macro_derive_resolution_fallback,
    pub_use_of_private_extern_crate,
    question_mark_macro_sep,
    safe_extern_statics,
    safe_packed_borrows,
    stable_features,
    trivial_bounds,
    trivial_casts,
    trivial_numeric_casts,
    type_alias_bounds,
    tyvar_behind_raw_pointer,
    unconditional_recursion,
    unions_with_drop_fields,
    unknown_crate_types,
    unnameable_test_items,
    unreachable_code,
    unreachable_patterns,
    unreachable_pub,
    unsafe_code,
    unstable_features,
    unstable_name_collisions,
    unused_allocation,
    unused_assignments,
    unused_attributes,
    unused_comparisons,
    unused_doc_comments,
    unused_extern_crates,
    unused_features,
    unused_import_braces,
    unused_imports,
    unused_labels,
    unused_lifetimes,
    unused_macros,
    unused_must_use,
    unused_mut,
    unused_parens,
    unused_qualifications,
    unused_unsafe,
    unused_variables,
    where_clauses_object_safety,
    while_true
)]
#![warn(unknown_lints)]
#![allow(
    bare_trait_objects,
    box_pointers,
    elided_lifetimes_in_paths,
    missing_debug_implementations,
    single_use_lifetimes,
    unused_results,
    variant_size_differences,
    warnings,
    renamed_and_removed_lints
)]
#![cfg_attr(feature = "clippy", feature(plugin))]

//! # simple_redis
//!
//! Simple and resilient [redis](https://redis.io/) client based on [redis-rs](https://crates.io/crates/redis) with
//! internal connection and subscription handling.
//!
//! This library provides a very basic, simple API for the most common redis operations.<br>
//! While not as comprehensive or flexiable as [redis-rs](https://crates.io/crates/redis),
//! it does provide a simpler api for most common use cases and operations as well as automatic and resilient internal
//! connection and subscription (pubsub) handling.<br>
//! In addition, the entire API is accessible via redis client and there is no need to manage connection or pubsub
//! instances in parallel.<br>
//!
//! ## Connection Resiliency
//!
//! Connection resiliency is managed by verifying the internally managed connection before every operation against the
//! redis server.<br>
//! In case of any connection issue, a new connection will be allocated to ensure the operation is invoked on a valid
//! connection only.<br>
//! However, this comes at a small performance cost of PING operation to the redis server.<br>
//!
//! ## Subscription Resiliency
//!
//! Subscription resiliency is ensured by recreating the internal pubsub and issuing new subscription requests
//! automatically in case of any error while fetching a message from the subscribed channels.
//!
//! # Examples
//!
//! ## Initialization and Simple Operations
//!
//! ```
//! extern crate simple_redis;
//!
//! fn main() {
//!     match simple_redis::create("redis://127.0.0.1:6379/") {
//!         Ok(mut client) =>  {
//!             println!("Created Redis Client");
//!
//!             match client.set("my_key", "my_value") {
//!                 Err(error) => println!("Unable to set value in Redis: {}", error),
//!                 _ => println!("Value set in Redis")
//!             };
//!
//!             match client.get_string("my_key") {
//!                 Ok(value) => println!("Read value from Redis: {}", value),
//!                 Err(error) => println!("Unable to get value from Redis: {}", error)
//!             };
//!
//!             match client.set("my_numeric_key", 255.5) {
//!                 Err(error) => println!("Unable to set value in Redis: {}", error),
//!                 _ => println!("Value set in Redis")
//!             };
//!
//!             match client.get::<f32>("my_numeric_key") {
//!                 Ok(value) => println!("Read value from Redis: {}", value),
//!                 Err(error) => println!("Unable to get value from Redis: {}", error)
//!             };
//!
//!             match client.hgetall("my_map") {
//!                 Ok(map) => {
//!                     match map.get("my_field") {
//!                         Some(value) => println!("Got field value from map: {}", value),
//!                         None => println!("Map field is emtpy"),
//!                     }
//!                 },
//!                 Err(error) => println!("Unable to read map from Redis: {}", error),
//!             };
//!
//!             /// run some command that is not built in the library
//!             match client.run_command::<String>("ECHO", vec!["testing"]) {
//!                 Ok(value) => assert_eq!(value, "testing"),
//!                 _ => panic!("test error"),
//!             };
//!
//!             /// publish messages
//!             let result = client.publish("news_channel", "test message");
//!             assert!(result.is_ok());
//!         },
//!         Err(error) => println!("Unable to create Redis client: {}", error)
//!     }
//! }
//! ```
//!
//! ## Subscription Flow
//!
//! ```rust,no_run
//! extern crate simple_redis;
//!
//! fn main() {
//!     match simple_redis::create("redis://127.0.0.1:6379/") {
//!         Ok(mut client) =>  {
//!             println!("Created Redis Client");
//!
//!             let mut result = client.subscribe("important_notifications");
//!             assert!(result.is_ok());
//!             result = client.psubscribe("*_notifications");
//!             assert!(result.is_ok());
//!
//!             loop {
//!                 // fetch next message (wait up to 5 seconds, 0 for no timeout)
//!                 match client.get_message(5000) {
//!                     Ok(message) => {
//!                         let payload: String = message.get_payload().unwrap();
//!                         assert_eq!(payload, "my important message")
//!                     },
//!                     Err(error) => println!("Error while fetching message, should retry again, info: {}", error),
//!                 }
//!             }
//!         },
//!         Err(error) => println!("Unable to create Redis client: {}", error)
//!     }
//! }
//! ```
//!
//! ## Closing Connection
//!
//! ```rust
//! extern crate simple_redis;
//!
//! fn main() {
//!     match simple_redis::create("redis://127.0.0.1:6379/") {
//!         Ok(mut client) =>  {
//!             println!("Created Redis Client");
//!
//!             match client.set("my_key", "my_value") {
//!                 Err(error) => println!("Unable to set value in Redis: {}", error),
//!                 _ => println!("Value set in Redis")
//!             };
//!
//!             match client.quit() {
//!                 Err(error) => println!("Error: {}", error),
//!                 _ => println!("Connection Closed.")
//!             }
//!         },
//!         Err(error) => println!("Unable to create Redis client: {}", error)
//!     }
//! }
//! ```
//!
//! # Installation
//! In order to use this library, just add it as a dependency:
//!
//! ```ini
//! [dependencies]
//! simple_redis = "*"
//! ```
//!
//! # Contributing
//! See [contributing guide](https://github.com/sagiegurari/simple_redis/blob/master/.github/CONTRIBUTING.md)
//!
//! # License
//! Developed by Sagie Gur-Ari and licensed under the
//! [Apache 2](https://github.com/sagiegurari/simple_redis/blob/master/LICENSE) open source license.
//!

#[cfg(test)]
#[path = "./lib_test.rs"]
mod lib_test;

pub mod client;
mod commands;
mod connection;
pub mod types;

/// Error Type
pub type RedisError = types::RedisError;

/// Error Info
pub type ErrorInfo = types::ErrorInfo;

/// PubSub message
pub type Message = types::Message;

/// Redis result which either holds a value or a Redis error
pub type RedisResult<T> = types::RedisResult<T>;

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
///
/// fn main() {
///     match simple_redis::create("redis://127.0.0.1:6379/") {
///         Ok(client) => println!("Created Redis Client"),
///         Err(error) => println!("Unable to create Redis client: {}", error)
///     }
/// }
/// ```
pub fn create(connection_string: &str) -> Result<client::Client, RedisError> {
    client::create(connection_string)
}
