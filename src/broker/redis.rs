use redis::Commands;

use crate::{command::Command, config::Config};

use super::Broker;

pub struct Redis {
    connection: redis::Connection,
    config: Config,
}

impl Redis {
    pub fn new(config: Config) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(config.redis_conn_str.to_string())?;
        let connection = client.get_connection()?;
        Ok(Self { connection, config })
    }
}

impl Broker for Redis {
    fn receive(&mut self) -> Option<Command<serde_json::Value>> {
        let message: String = match self
            .connection
            .zpopmin::<std::string::String, (std::string::String, std::string::String)>(
                self.config.redis_queue.clone(),
                self.config.redis_consume_batch_size,
            ) {
            Err(_) => return None,
            Ok(msg) => msg.0,
        };
        Command::try_from(message).ok()
    }
}

#[cfg(test)]
mod tests {
    use crate::config;

    use super::*;

    #[test]
    fn given_config_when_initialize_redis_then_connection() {
        let conf = Config {
            environment: config::Environment::Local,
            redis_conn_str: String::from("redis://localhost:6379"),
            redis_queue: String::from("queue"),
            redis_consume_batch_size: 1,
        };
        Redis::new(conf).unwrap();
    }

    #[test]
    fn given_bad_config_when_initialize_redis_then_error() {
        let conf = Config {
            environment: config::Environment::Local,
            redis_conn_str: String::from("nope"),
            redis_queue: String::from("queue"),
            redis_consume_batch_size: 1,
        };
        assert!(Redis::new(conf).is_err());
    }

    #[test]
    fn given_empty_redis_when_receive_then_empty() {
        let conf = Config {
            environment: config::Environment::Local,
            redis_conn_str: String::from("redis://localhost:6379"),
            redis_queue: String::from("queue"),
            redis_consume_batch_size: 1,
        };
        let mut redis = Redis::new(conf).unwrap();
        assert!(redis.receive().is_none());
    }

    #[test]
    fn given_loaded_redis_when_receive_then_job() {
        let json: &str = "
        {
            \"command_type\": 10,
            \"payload\": \"\"
        }";
        let conf = Config {
            environment: config::Environment::Local,
            redis_conn_str: String::from("redis://localhost:6379"),
            redis_queue: String::from("queue"),
            redis_consume_batch_size: 1,
        };
        let mut r: redis::Client = redis::Client::open(conf.redis_conn_str.clone()).unwrap();
        r.get_connection().unwrap();
        r.zadd::<std::string::String, isize, std::string::String, isize>(
            conf.redis_queue.clone(),
            String::from(json),
            10,
        )
        .unwrap();
        let mut redis = Redis::new(conf).unwrap();
        assert_eq!(redis.receive().unwrap().command_type, 10);
    }
}
