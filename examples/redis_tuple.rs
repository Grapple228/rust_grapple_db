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
    let data1: [u8; 4] = [0, 1, 2, 3];
    let data2: [u8; 4] = [4, 5, 6, 7];
    let tuple1 = (key1, &data1);
    let tuple2 = (key2, &data2);

    // (String, i32) тоже работает
    client.set(&("3213".to_string(), 3)).await?;

    // Читаем i32 через FromRedisValue напрямую
    let val: Option<i32> = client.get("3213").await?;
    println!("{:?}", val);

    // -- Set tuples
    assert_eq!("OK", client.mset([&tuple1, &tuple2]).await?);
    assert!(!client.mset_nx([&tuple1, &tuple2]).await?);

    // -- Check exists
    assert!(client.exists(&tuple1.0).await?);
    assert!(client.exists(&tuple2.0).await?);

    // -- Read tuples
    println!("Get tuples");

    // Читаем как Vec<u8> через FromRedisValue
    let got_values: Vec<Option<[u8; 4]>> = client.mget(&[&tuple1.0, &tuple2.0]).await?;
    println!("Multiple {:?}", got_values);

    let get1: Vec<u8> = client.get(&tuple1.0).await?.unwrap();
    println!("Tuple 1:  {:?}", get1);

    let get2: Vec<u8> = client.get(&tuple2.0).await?.unwrap();
    println!("Tuple 2:  {:?}", get2);

    println!();

    // -- Del tuples
    println!("Del tuples");

    assert_eq!(2, client.mdel([&tuple1.0, &tuple2.0]).await?);

    println!();

    // -- Check exists
    assert!(!client.exists(&tuple1.0).await?);
    assert!(!client.exists(&tuple2.0).await?);

    // -- Read model after delete
    println!("Get after delete");

    let get1: Option<Vec<u8>> = client.get(&tuple1.0).await?;
    println!("Tuple 1: {:?}", get1);

    let get2: Option<Vec<u8>> = client.get(&tuple2.0).await?;
    println!("Tuple 2: {:?}", get2);

    Ok(())
}

#[cfg(not(feature = "redis"))]
fn main() {
    panic!("This example requires 'redis' enabled feature")
}
