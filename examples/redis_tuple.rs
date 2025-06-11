#[cfg(feature = "redis")]
use grapple_db::redis::Client;

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For early dev.

#[cfg(feature = "redis")]
#[tokio::main]
async fn main() -> Result<()> {
    // -- Init client
    let client = Client::default().await?;

    let key1 = "tuple1".to_string();
    let key2 = "tuple2".to_string();

    // -- Create tuples
    let tuple1 = (key1.clone(), "my tuple 1".to_string());
    let tuple2 = (key2.clone(), "my tuple 2".to_string());

    client.set(&("3213".to_string(), 3)).await?;

    println!("{:?}", client.get::<i32>("3213").await?);

    // -- Set tuples
    assert_eq!("OK", client.mset(&[&tuple1, &tuple2]).await?);
    assert!(!client.mset_nx(&[&tuple1, &tuple2]).await?);

    // -- Check exists
    assert!(client.exists(&tuple1.0).await?);
    assert!(client.exists(&tuple2.0).await?);

    // -- Read tuples
    println!("Get tuples");

    let got_values: Vec<Option<String>> = client.mget([&key1, &key2]).await?;
    println!("Multiple {:?}", got_values);

    let get1: String = client.get(&key1).await?.unwrap();
    println!("Tuple 1:  {}", get1);

    let get2: String = client.get(&key2).await?.unwrap();
    println!("Tuple 2:  {}", get2);

    println!();

    // -- Del tuples
    println!("Del tuples");

    assert_eq!(2, client.mdel([&key1, &key2]).await?);

    println!();

    // -- Check exists
    assert!(!client.exists(&key1).await?);
    assert!(!client.exists(&key2).await?);

    // -- Read model after delete
    println!("Get after delete");

    let get1: Option<String> = client.get(&key1).await?;
    println!("Tuple 1: {:?}", get1);

    let get2: Option<String> = client.get(&key2).await?;
    println!("Tuple 2: {:?}", get2);

    Ok(())
}

#[cfg(not(feature = "redis"))]
fn main() {
    panic!("This example requires 'redis' enabled feature")
}
