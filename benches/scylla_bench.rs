#![allow(unused)]

#[cfg(feature = "scylla")]
use charybdis::{
    batch::ModelBatch,
    macros::{charybdis_model, charybdis_udt_model},
    operations::{Delete, Insert, Update},
    types::{Boolean, Text, Timestamp, Uuid},
};

#[cfg(feature = "scylla")]
use scylla::{
    client::{caching_session::CachingSession, session_builder::SessionBuilder},
    errors::NextRowError,
    statement::batch::{Batch, BatchStatement, BatchType},
};

#[cfg(feature = "scylla")]
use grapple_db::scylla::Client;

use chrono::Utc;
use criterion::Throughput;
use criterion::{criterion_group, criterion_main, Criterion};
use std::sync::Arc;
use tokio::runtime::Runtime;

const NODES: [&str; 3] = ["127.0.0.1:9042", "127.0.0.1:9043", "127.0.0.1:9044"];
const KEYSPACE: &str = "benchmarks";
const CACHE_SIZE: usize = 1000;

#[cfg(feature = "scylla")]
#[charybdis_model(
    table_name = bench_users,
    partition_keys = [id],
    clustering_keys = []
)]
#[derive(Clone)]
pub struct BenchUser {
    pub id: Uuid,
    pub username: Text,
    pub email: Text,
    pub created_at: Timestamp,
}

#[cfg(feature = "scylla")]
#[derive(Default, Clone)]
#[charybdis_udt_model(type_name = profile)]
pub struct Profile {
    pub first_name: Text,
    pub last_name: Text,
    pub username: Text,
    pub email: Text,
}

#[cfg(feature = "scylla")]
impl Profile {
    pub fn sample() -> Self {
        Self {
            first_name: "Homer".to_string(),
            last_name: "Simpson".to_string(),
            username: "homer".to_string(),
            email: "homer@simpson.com".to_string(),
        }
    }
}

#[cfg(feature = "scylla")]
#[charybdis_model(
    table_name = posts,
    partition_keys = [community_id],
    clustering_keys = [created_at, id],
    global_secondary_indexes = [],
    table_options = "
        CLUSTERING ORDER BY (created_at DESC)
    "
)]
#[derive(Default, Clone)]
pub struct Post {
    pub community_id: Uuid,
    pub created_at: Timestamp,
    pub id: Uuid,
    pub title: Text,
    pub description: Text,
    pub updated_at: Timestamp,
    pub creator_id: Uuid,
    pub creator: Profile,
    pub is_archived: Boolean,
}

#[cfg(feature = "scylla")]
impl Post {
    pub fn sample() -> Post {
        Post {
            community_id: Uuid::new_v4(),
            created_at: Utc::now(),
            id: Uuid::new_v4(),
            title: "Test".to_string(),
            description: "Test".to_string(),
            updated_at: Default::default(),
            creator_id: Uuid::new_v4(),
            creator: Profile::sample(),
            is_archived: false,
        }
    }

    pub async fn populate_sample_posts_per_partition(
        community_id: Uuid,
        db_session: &CachingSession,
    ) {
        let mut posts = vec![];
        for i in 0..10000 {
            let mut post = Post::sample();
            post.community_id = community_id;
            post.title = format!("Post {}", i);
            post.description = format!("Post {}", i);
            post.creator_id = Uuid::new_v4();
            posts.push(post);
        }

        Post::batch()
            .chunked_insert(db_session, &posts, 10000)
            .await
            .expect("Failed to insert posts");
    }
}

#[cfg(feature = "scylla")]
async fn create_keyspace() {
    let create_keyspace_query = format!(
        r#"
            CREATE KEYSPACE IF NOT EXISTS {}
            WITH REPLICATION = {{
                'class': 'SimpleStrategy',
                'replication_factor': 1
            }}
        "#,
        KEYSPACE
    );

    let session: CachingSession = CachingSession::from(
        SessionBuilder::new()
            .known_nodes(NODES)
            .build()
            .await
            .expect("Unable to connect to scylla hosts"),
        CACHE_SIZE,
    );

    session
        .execute_unpaged(create_keyspace_query, ())
        .await
        .unwrap();
}

#[cfg(feature = "scylla")]
async fn setup_database() -> CachingSession {
    create_keyspace().await;

    let session: CachingSession = CachingSession::from(
        SessionBuilder::new()
            .use_keyspace(KEYSPACE, false)
            .known_nodes(NODES)
            .build()
            .await
            .expect("Unable to connect to scylla hosts"),
        CACHE_SIZE,
    );
    let create_user_table = "
        CREATE TABLE IF NOT EXISTS bench_users (
            id UUID PRIMARY KEY,
            username TEXT,
            email TEXT,
            created_at TIMESTAMP
        )
    ";

    let create_post_table = "
        CREATE TABLE IF NOT EXISTS posts (
            community_id UUID,
            created_at TIMESTAMP,
            id UUID,
            title TEXT,
            description TEXT,
            updated_at TIMESTAMP,
            creator_id UUID,
            creator Profile,
            is_archived BOOLEAN,
            PRIMARY KEY (community_id, created_at, id)
        )
    ";

    let create_profile_udt = "
        CREATE TYPE IF NOT EXISTS profile (
            first_name TEXT,
            last_name TEXT,
            username TEXT,
            email TEXT
        )
    ";
    session
        .execute_unpaged(create_profile_udt, ())
        .await
        .unwrap();
    session
        .execute_unpaged(create_user_table, ())
        .await
        .unwrap();
    session
        .execute_unpaged(create_post_table, ())
        .await
        .unwrap();

    session
}

#[cfg(feature = "scylla")]
async fn drop_keyspace() {
    let drop_keyspace_query = format!("DROP KEYSPACE IF EXISTS {}", KEYSPACE);

    let session: CachingSession = CachingSession::from(
        SessionBuilder::new()
            .known_nodes(NODES)
            .build()
            .await
            .expect("Unable to connect to scylla hosts"),
        CACHE_SIZE,
    );

    session
        .execute_unpaged(drop_keyspace_query, ())
        .await
        .unwrap();
}

#[cfg(feature = "scylla")]
fn bench_orm_vs_native(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let session = rt.block_on(async { setup_database().await });
    let session = Arc::new(session);

    let client = Client::from_session(&session).unwrap();

    let test_user = BenchUser {
        id: Uuid::new_v4(),
        username: "charybdis".to_string(),
        email: "charybdis@orm.com".to_string(),
        created_at: Utc::now(),
    };

    let category_id: Uuid = Uuid::new_v4();
    rt.block_on(async {
        Post::populate_sample_posts_per_partition(category_id, &session.clone()).await
    });

    // Benchmark Insert
    c.benchmark_group("Insert Benchmarks")
        .throughput(Throughput::Elements(1))
        .bench_function("ORM Insert", |b| {
            b.iter(|| {
                rt.block_on(async {
                    test_user.insert().execute(&session).await.unwrap();
                });
            });
        })
        .bench_function("My Insert", |b| {
            b.iter(|| {
                rt.block_on(async {
                    client.insert(&test_user).await.unwrap();
                })
            });
        })
        .bench_function("Native Insert", |b| {
            b.iter(|| {
                rt.block_on(async {
                    session
                        .execute_unpaged(
                            "INSERT INTO bench_users (id, username, email, created_at) VALUES (?, ?, ?, ?)",
                            (test_user.id, "charybdis", "charybdis@orm.com", test_user.created_at),
                        )
                        .await
                        .unwrap();
                });
            });
        })
        ;

    c.benchmark_group("ORM Batch Insert")
        .throughput(Throughput::Elements(1000))
        .bench_function("ORM Batch Insert", |b| {
            b.iter(|| {
                let orm_users = (0..1000)
                    .map(|_| BenchUser {
                        id: Uuid::new_v4(),
                        username: "charybdis".to_string(),
                        email: "charybdis@email.com".to_string(),
                        created_at: Utc::now(),
                    })
                    .collect::<Vec<BenchUser>>();

                rt.block_on(async {
                    BenchUser::batch()
                        .chunked_insert(&session, &orm_users, 1000)
                        .await
                        .unwrap();
                });
            });
        })
        .bench_function("My BatchInsert", |b| {
            b.iter(|| {
                let orm_users = (0..1000)
                    .map(|_| BenchUser {
                        id: Uuid::new_v4(),
                        username: "charybdis".to_string(),
                        email: "charybdis@email.com".to_string(),
                        created_at: Utc::now(),
                    })
                    .collect::<Vec<BenchUser>>();

                rt.block_on(async {
                    client.insert_many(&orm_users, 1000).await.unwrap();
                })
            });
        })
        .bench_function("Native Batch Insert", |b| {
            b.iter(|| {
                rt.block_on(async {
                    let native_insert_statements = (0..1000)
                        .map(|_| {
                            BatchStatement::Query(scylla::statement::unprepared::Statement::new(
                                "INSERT INTO bench_users (id, username, email, created_at) VALUES (?, ?, ?, ?)",
                            ))
                        })
                        .collect::<Vec<BatchStatement>>();

                    let native_values = (0..1000)
                        .map(|_| (Uuid::new_v4(), "charybdis", "test@mail.com", Utc::now()))
                        .collect::<Vec<(Uuid, &str, &str, Timestamp)>>();

                    let batch = Batch::new_with_statements(BatchType::Logged, native_insert_statements);
                    session.batch(&batch, native_values).await.unwrap();
                });
            });
        });

    // Benchmark Find
    c.bench_function("ORM Find", |b| {
        b.iter(|| {
            rt.block_on(async {
                BenchUser::find_by_id(test_user.id)
                    .execute(&session)
                    .await
                    .unwrap();
            });
        });
    });

    c.bench_function("My Find", |b| {
        b.iter(|| {
            rt.block_on(async {
                client
                    .get(BenchUser::find_by_id(test_user.id))
                    .await
                    .unwrap();
            })
        });
    });

    c.bench_function("Native Find", |b| {
        b.iter(|| {
            rt.block_on(async {
                let res = session
                    .execute_unpaged(
                        "SELECT id, username, email, created_at FROM bench_users WHERE id = ?",
                        (test_user.id,),
                    )
                    .await
                    .unwrap();

                let res = res.into_rows_result().unwrap();

                res.first_row::<(Uuid, &str, &str, Timestamp)>().unwrap();
            });
        });
    });

    c.bench_function("ORM Stream - Find Posts Per Partition", |b| {
        b.iter(|| {
            rt.block_on(async {
                Post::find_by_community_id(category_id)
                    .execute(&session)
                    .await
                    .unwrap()
                    .try_collect()
                    .await
                    .unwrap();
            });
        });
    });

    c.bench_function("My Stream - Find Posts Per Partition", |b| {
        b.iter(|| {
            rt.block_on(async {
                client
                    .stream(Post::find_by_community_id(category_id))
                    .await
                    .unwrap()
                    .try_collect()
                    .await
                    .unwrap();
            })
        });
    });

    c.bench_function("Native Stream - Find Posts Per Partition", |b| {
        b.iter(|| {
            rt.block_on(async {
                use futures::TryStreamExt;

                let res = session
                    .execute_iter("SELECT * FROM posts WHERE community_id = ?", (category_id,))
                    .await
                    .unwrap()
                    .rows_stream::<Post>()
                    .unwrap();

                let results: Result<Vec<Post>, NextRowError> = res.try_collect().await;
                results.unwrap();
            });
        });
    });

    // Benchmark Update
    let updated_user = BenchUser {
        username: "updated_charybdis".to_string(),
        ..test_user.clone()
    };

    c.bench_function("ORM Update", |b| {
        b.iter(|| {
            rt.block_on(async {
                updated_user.update().execute(&session).await.unwrap();
            });
        });
    });

    c.bench_function("My Update", |b| {
        b.iter(|| {
            rt.block_on(async {
                client.update(&updated_user).await.unwrap();
            })
        });
    });

    c.bench_function("Native Update", |b| {
        b.iter(|| {
            rt.block_on(async {
                session
                    .execute_unpaged(
                        "UPDATE bench_users SET username = ? WHERE id = ?",
                        ("updated_charybdis", updated_user.id),
                    )
                    .await
                    .unwrap();
            });
        });
    });

    // Benchmark Delete
    c.bench_function("ORM Delete", |b| {
        b.iter(|| {
            rt.block_on(async {
                test_user.delete().execute(&session).await.unwrap();
            });
        });
    });

    c.bench_function("My Delete", |b| {
        b.iter(|| {
            rt.block_on(async {
                client.delete(&test_user).await.unwrap();
            })
        });
    });

    c.bench_function("Native Delete", |b| {
        b.iter(|| {
            rt.block_on(async {
                session
                    .execute_unpaged("DELETE FROM bench_users WHERE id = ?", (test_user.id,))
                    .await
                    .unwrap();
            });
        });
    });

    rt.block_on(async { drop_keyspace().await });
}

#[cfg(not(feature = "scylla"))]
fn main() {
    panic!("This example requires 'sqylla' enabled feature")
}

#[cfg(feature = "scylla")]
criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(100);
    targets = bench_orm_vs_native
}

#[cfg(feature = "scylla")]
criterion_main!(benches);
