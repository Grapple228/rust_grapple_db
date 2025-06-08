// region:    --- Modules

#[cfg(feature = "scylla")]
use grapple_db::scylla::charybdis::{
    macros::charybdis_model,
    types::{Text, Uuid},
};
#[cfg(feature = "scylla")]
use grapple_db::{
    scylla::{stream::PagableCharybdisStream, Client, ConnectionParams, CrudParams},
    Pagable,
};

// endregion: --- Modules

// region:    --- Model

#[cfg(feature = "scylla")]
#[charybdis_model(
    table_name = users,
    partition_keys = [id],
    clustering_keys = [],

    global_secondary_indexes = [name],
    local_secondary_indexes = [],
)]
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct User {
    id: Uuid,
    name: Option<Text>,

    pwd: Option<Text>,
}

// endregion: --- Model

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For early dev.

#[cfg(not(feature = "scylla"))]
fn main() {
    panic!("This example requires 'sqylla' enabled feature")
}

#[cfg(feature = "scylla")]
#[tokio::main]
async fn main() -> Result<()> {
    // -- INIT CLIENT

    let con_params = ConnectionParams {
        fetch_keyspaces: ["auth".into()].to_vec(),
        use_keyspace: Some("auth".into()),
        migrate: true,
        recreate_keyspace: true,
        ..Default::default()
    };

    let client = Client::connect(&con_params)
        .await?
        .with_params(CrudParams::default());

    // -- SEED DB
    let custom = seed_users(&client).await?;

    // -- UPDATE USER
    let mut custom_changed: User = custom.clone();
    custom_changed.name = Some("Other name".into());

    client.update(&custom_changed).await?;

    // Check
    let changed_check = client.get(User::find_by_id(custom.id)).await?;
    assert_eq!(changed_check.name, custom_changed.name);

    // -- DELETE USER
    client.delete(&changed_check).await?;
    assert_eq!(changed_check.name, custom_changed.name);

    let deleted_check = client.get(User::find_by_id(custom.id)).await;
    assert!(deleted_check.is_err());

    // -- GET COUNT
    let _my_count = client.count(User::find_by_name("find me".into())).await?;

    // -- STREAM USERS
    let mut users_count = 0;

    let stream = client
        .stream(User::find_by_name("find me".to_string()))
        // .stream(User::find_all())
        .await?;

    let mut pagable_stream = PagableCharybdisStream::new(stream, 5);

    while let Some(users) = pagable_stream.next_page().await {
        for user in users {
            println!("{} {:?} {:?}", user.id, user.name, user.pwd);
        }

        users_count += &users.len();
    }

    println!("Users in stream: {}", users_count);

    Ok(())
}

#[cfg(feature = "scylla")]
async fn seed_users(client: &Client) -> Result<User> {
    use std::time::Instant;

    const ITEMS_COUNT: usize = 10_000;
    const CHUNK_SIZE: usize = 3000;
    const FIND_COUNT: usize = 100;

    let mut users = Vec::new();

    // -- Create users

    // Custom user
    let custom_id = Uuid::new_v4();
    let custom = User {
        id: custom_id,
        name: Some("Custom user".into()),
        pwd: Some("my pwd".into()),
    };
    users.push(custom.clone());

    // Users for finding
    for _ in 0..FIND_COUNT {
        let user = User {
            id: Uuid::new_v4(),
            name: Some("find me".into()),
            ..Default::default()
        };

        users.push(user);
    }

    // Other users
    for i in 0..ITEMS_COUNT {
        let user = User {
            id: Uuid::new_v4(),
            name: Some(format!("User {i}")),
            ..Default::default()
        };

        users.push(user);
    }

    // -- Insers users
    let start = Instant::now();
    client.insert_many(&users, CHUNK_SIZE).await?;
    let end = start.elapsed();
    println!(
        "Create many (batch) {} elements: {:?} with chunk_size: {}",
        ITEMS_COUNT + 3,
        end,
        CHUNK_SIZE
    );

    Ok(custom)
}
