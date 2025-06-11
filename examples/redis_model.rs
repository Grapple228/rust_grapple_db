#[cfg(feature = "redis")]
use grapple_db::redis::{self, macros::FromRedisValue, Client, RedisModel};
#[cfg(feature = "redis")]
use serde::{Deserialize, Serialize};

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For early dev.

// region:    --- Model Definition

#[cfg(feature = "redis")]
#[derive(Debug, Serialize, Deserialize, FromRedisValue)]
pub struct Model {
    a: i32,
    b: i32,
}

#[cfg(feature = "redis")]
impl RedisModel for Model {
    fn key(&self) -> redis::Result<String> {
        // Key for model
        Ok(format!("{}.{}", self.a, self.b))
    }
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

    // -- Get keys
    let key1 = model1.key()?;
    let key2 = model2.key()?;

    // -- Set models
    assert_eq!("OK", client.mset(&[&model1, &model2]).await?);
    assert!(!client.mset_nx(&[&model1, &model2]).await?);

    // -- Check exists
    assert!(client.exists(&key1).await?);
    assert!(client.exists(&key2).await?);

    // -- Read model
    println!("Get models");

    let got_models: Vec<Option<Model>> = client.mget(&[&key1, &key2]).await?;
    println!("Multiple {:?}", got_models);

    let get1: Model = client.get(&key1).await?.unwrap();
    println!("Model 1:  {:?}", get1);

    let get2: Model = client.get(&key2).await?.unwrap();
    println!("Model 2:  {:?}", get2);

    println!();

    // -- Del model
    println!("Del models");

    assert_eq!(2, client.mdel([&key1, &key2]).await?);

    println!();

    // -- Check exists
    assert!(!client.exists(&key1).await?);
    assert!(!client.exists(&key2).await?);

    // -- Read model after delete
    println!("Get after delete");

    let get1: Option<Model> = client.get(&key1).await?;
    println!("Model 1: {:?}", get1);

    let get2: Option<Model> = client.get(&key2).await?;
    println!("Model 2: {:?}", get2);

    Ok(())
}

#[cfg(not(feature = "redis"))]
fn main() {
    panic!("This example requires 'redis' enabled feature")
}
