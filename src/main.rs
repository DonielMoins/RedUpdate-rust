#![feature(result_option_inspect)]
// use futures::prelude::*;
// use async_std::io;
// use async_std::prelude::*;
// use std::future::Future;

use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::{broadcast, Mutex};
use tokio::{task, time};
use warp::http::Error;
use warp::hyper::Method;
use warp::Filter;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

extern crate redis;
use redis::Commands;
type Db = Arc<Mutex<HashMap<String, String>>>;

#[tokio::main]
async fn main() {
    // loop {
    //     print!("test");
    // }
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    let tx: Sender<String> = broadcast::channel(512).0;

    println!("Started!");
    let _keyevent_handler = tokio::spawn(get_match(db.clone(), tx.clone()));
    let _keyevent_db_handler = tokio::spawn(key_value_change_handler(
        db.clone(),
        tx.subscribe(),
        tx.clone(),
    ));

    let _webserver = tokio::spawn(restful_redis(db.clone()));

    // Always last to run.
    let _debug_printclient = task::spawn(print_db_loop(db.clone(), tx.subscribe())).await;
}

async fn restful_redis(db: Db) -> Result<(), Error> {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec![
            "Access-Control-Allow-Headers",
            "Access-Control-Request-Method",
            "Access-Control-Request-Headers",
            "Access-Control-Allow-Origin",
            "Accept",
            "X-Requested-With",
            "Content-Type",
            "Accept-Encoding",
            "Accept-Language",
            "User-Agent",
            "Sec-Fetch-Mode",
            "Referer",
            "Origin",
            "DNT",
            "Host",
            "Sec-Fetch-Dest",
        ])
        .allow_methods(vec!["POST", "GET"])
        .build();

    let routes = warp::any()
        .map(move || {
            // let db = db.clone().blocking_lock().values().collect();
            let json = serde_json::json!((&db.clone().try_lock().as_deref().unwrap()));
            format!("{}", json.to_string())
        })
        .with(cors);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    Ok(())
}

async fn create_client() -> redis::RedisResult<redis::Connection> {
    println!("Attempting to connect to Redis Server...");
    let client = redis::Client::open("redis://127.0.0.1/")?;

    println!("Establishing Client connection");
    Ok(client.get_connection()?)
}

async fn get_match(db: Db, tx: Sender<String>) -> redis::RedisResult<()> {
    let res_con = create_client().await;
    if res_con.is_err() {
        println!("Error: Client could not establish connection!");
        return Ok(());
    }
    let mut con = res_con.unwrap();
    println!("Trying 'CONFIG SET KEA' to be able to subscribe to Keyspace events. Refer to: https://redis.io/docs/manual/keyspace-notifications/");

    let ev = redis::cmd("config")
        .arg("set")
        .arg("notify-keyspace-events")
        .arg("sEA")
        .query(&mut con)?;
    let _ = println!("{:#?}", ev);

    let mut pubsub = con.as_pubsub();
    println!("Created Pubsub");
    pubsub.psubscribe("*")?;

    println!("Looping now!");
    loop {
        let msg = pubsub.get_message()?;
        let payload: String = msg.get_payload()?;
        // println!("channel '{}': {}", msg.get_channel_name(), payload);
        let mut db = db.lock().await;
        db.insert(payload.clone(), "NULL_VALUE".to_string());
        tx.send(String::from(format!(
            "{}{}",
            "db_update:",
            payload.clone().as_str()
        )))
        .inspect_err(|e| eprint!("Failed to send db_update: {e}."))
        .ok();
    }
}

// Handles updates for recently modifed keys.
async fn key_value_change_handler(
    db: Db,
    mut rx: Receiver<String>,
    tx: Sender<String>,
) -> redis::RedisResult<()> {
    let mut interval = time::interval(Duration::from_millis(20));

    let res_con = create_client().await;
    if res_con.is_err() {
        println!("Error: Client could not establish connection!");
        return Ok(());
    }
    let mut con = res_con.unwrap();

    let mut start_keys: Vec<_> = vec![];
    con.scan()
        .and_then(|keys: redis::Iter<String>| Ok(start_keys = keys.collect()));
    for key in start_keys {
        let val: Option<String> = con.get(key.clone())?;
        let mut db = db.lock().await;
        db.insert(key.to_string(), val.clone().unwrap().to_string());
    }
    tx.send("db_ready".to_string());
    loop {
        interval.tick().await;
        if !rx.is_empty() {
            let mut msg = rx.recv().await.unwrap();
            msg.make_ascii_lowercase();
            if msg.starts_with("db_update:") {
                let key = msg.clone().replace("db_update:", "");
                let val: Option<String> = con.get(key.clone())?;
                let mut db = db.lock().await;
                db.insert(key.to_string(), val.clone().unwrap().to_string());
                tx.send("db_ready".to_string());
            }
        }
    }
}

async fn print_db_loop(db: Db, mut rx: Receiver<String>) {
    let mut interval = time::interval(Duration::from_millis(100));
    loop {
        interval.tick().await;
        if !rx.is_empty() && String::from(rx.recv().await.unwrap()).starts_with("db_ready") {
            let db = db.lock().await;
            println!("\n---------------------------------------------\n{:#?}\n---------------------------------------------", db.iter());
        }
    }
}
