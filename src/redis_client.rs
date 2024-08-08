// src/redis_client.rs

use redis::Client;
use std::sync::Arc;

pub type RedisClient = Arc<Client>;

pub fn create_redis_client() -> RedisClient {
	let client = Client::open("redis://127.0.0.1/").expect("Invalid Redis URL");
	Arc::new(client)
}