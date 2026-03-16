use grapple_db::redis;
use grapple_db::redis::Client;
use grapple_redis_macros::FromRedisValue;

#[cfg(feature = "redis")]
#[cfg(feature = "redis")]
use serde::{Deserialize, Serialize};

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

// region:    --- Model Definition

#[cfg(feature = "redis")]
#[derive(Debug, Serialize, Deserialize, FromRedisValue, PartialEq)]
pub struct Model {
    a: i32,
    b: i32,
}

#[cfg(feature = "redis")]
impl Model {
    pub fn new(value: i32) -> Self {
        Self {
            a: 2 * value,
            b: 3 * value,
        }
    }
}
// endregion: --- Model Definition

#[cfg(feature = "redis")]
#[tokio::main]
async fn main() -> Result<()> {
    // -- Init client
    let client = Client::default().await?;

    // -- Create model
    let model1 = Model::new(1);
    let model2 = Model::new(2);

    // -- Создаем кортежи (key, value) для записи
    let key1 = "model:1".to_string();
    let key2 = "model:2".to_string();

    let tuple1 = (key1.clone(), serde_json::to_string(&model1)?);
    let tuple2 = (key2.clone(), serde_json::to_string(&model2)?);

    // -- Set models через tuple
    assert_eq!("OK", client.mset([&tuple1, &tuple2]).await?);
    assert!(!client.mset_nx([&tuple1, &tuple2]).await?);

    // -- Check exists
    assert!(client.exists(&key1).await?);
    assert!(client.exists(&key2).await?);

    // -- Read model
    println!("Get models");

    let got_models: Vec<Option<String>> = client.mget(&[&key1, &key2]).await?;
    println!("Raw JSON: {:?}", got_models);

    // Десериализуем вручную
    let get1: Model = serde_json::from_str(&client.get::<String, _>(&key1).await?.unwrap())?;
    let get2: Model = serde_json::from_str(&client.get::<String, _>(&key2).await?.unwrap())?;

    println!("Model 1:  {:?}", get1);
    println!("Model 2:  {:?}", get2);

    println!();

    // -- Del model
    println!("Del models");

    assert_eq!(2, client.mdel([&key1, &key2]).await?);

    // -- Check exists
    assert!(!client.exists(&key1).await?);
    assert!(!client.exists(&key2).await?);

    Ok(())
}

#[cfg(not(feature = "redis"))]
fn main() {
    panic!("This example requires 'redis' enabled feature")
}
