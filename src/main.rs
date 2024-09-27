mod auth;
mod libs;
mod users;

use auth::handler::login_user;
use deadpool_postgres::{Manager, Pool};
use governor::clock::QuantaClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use libs::{
    authenticate, get_db_url, get_worker_num, CORS_ALLOW_ALL, NOT_FOUND, TOO_MANY_REQUEST,
    UNAUTHORIZED,
};
use log::{debug, error, info, warn};
use std::io::Write;
use std::num::{NonZeroU32, NonZeroUsize};
use std::str::FromStr;
use std::sync::atomic::AtomicI32;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration};
use tokio_postgres::{Config, NoTls};
use users::handler::{create_user, delete_user, edit_user, get_user, list_user};

#[macro_use]
extern crate serde_derive;

struct AppState {
    db_pool: Pool,
    common_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, QuantaClock>>,
    hard_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, QuantaClock>>,
}

#[tokio::main]
async fn main() {
    // Setup database connection pool once and share it across handlers
    let db_url = get_db_url();
    let cfg = Config::from_str(&db_url).expect("Failed to parse DATABASE_URL");
    let manager = Manager::new(cfg, NoTls);
    let worker_num = get_worker_num();
    let pool = Pool::new(manager, 16); // 16 adalah ukuran maksimum pool

    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .format(|buf: &mut env_logger::fmt::Formatter, record| {
            writeln!(
                buf,
                "[{}] {}:{} - {}: {}",
                buf.timestamp(),
                record.file().unwrap_or("<unknown>"),
                record.line().unwrap_or(0),
                record.level(),
                record.args()
            )
        })
        .init();

    //start server and print port
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind to address 0.0.0.0:8080");
    let global_limiter = Arc::new(RateLimiter::direct(Quota::per_second(
        NonZeroU32::new(200 * worker_num as u32).unwrap(),
    )));
    let common_limiter = Arc::new(RateLimiter::direct(Quota::per_second(
        NonZeroU32::new(100).unwrap(),
    )));
    let hard_limiter = Arc::new(RateLimiter::direct(Quota::per_second(
        NonZeroU32::new(100).unwrap(),
    )));
    info!("Server listening on port 8080");

    let parallelism = std::thread::available_parallelism().map_or(2, NonZeroUsize::get);
    let tokio_default_max_blocking_thread = 512;
    let max_blocking_threads = std::cmp::max(tokio_default_max_blocking_thread / parallelism, 1);

    // Share AppState with all incoming connections
    let app_state = Arc::new(AppState {
        db_pool: pool,
        common_limiter: common_limiter.clone(),
        hard_limiter: hard_limiter.clone(),
    });

    let shutdown_signal = shutdown_signal_listener().await;

    // Create Worker Threads with new async runtime
    let mut txs = Vec::with_capacity(worker_num);
    let mut worker_sets = Vec::with_capacity(worker_num);
    for _ in 0..worker_num {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let semaphore = Arc::new(Semaphore::new(10));

        txs.push(tx);
        worker_sets.push((semaphore, rx));
    }

    for _ in 0..worker_num {
        let (semaphore, mut rx) = worker_sets.pop().unwrap();
        let shutdown_signal = shutdown_signal.clone();
        let global_limiter = global_limiter.clone();
        let app_state = app_state.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .max_blocking_threads(max_blocking_threads)
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime");

            rt.block_on(async {
                let mut shutdown_listener = shutdown_signal.subscribe();
                let counter = AtomicI32::new(0);

                loop {
                    tokio::select! {
                        Some(mut stream) = rx.recv() => {
                            let permit = semaphore
                                .clone()
                                .acquire_owned()
                                .await
                                .expect("Failed to acquire semaphore permit");

                            let state = app_state.clone();
                            let global_limiter = global_limiter.clone();
                            counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                            tokio::spawn(async move {
                                loop {
                                    match global_limiter.check() {
                                        Ok(()) => {
                                            handle_client(&mut stream, state).await;
                                            break;
                                        }
                                        Err(_) => sleep(Duration::from_millis(10)).await,
                                    }
                                }
                                drop(permit);
                            });
                        }
                        _ = shutdown_listener.recv() => {
                            info!("worker processed: {:?}", counter.load(std::sync::atomic::Ordering::Relaxed));
                            rx.close();

                            break;
                        }
                    }
                }
            })
        });
    }

    info!("started {} Workers", worker_num);

    let mut shutdown_listener = shutdown_signal.subscribe();
    let mut rrb_index = 0;
    let mut counter = 0;

    loop {
        tokio::select! {
            Ok((stream, _)) = listener.accept() => {
                let _ = txs[rrb_index].send(stream);
                rrb_index = (rrb_index + 1) % worker_num;

                counter += 1;
            }
            _ = shutdown_listener.recv() => {
                warn!("Shutting down system");
                sleep(Duration::from_secs(worker_num as u64)).await;
                info!("Total request served: {}", counter);
                break;
            }
        }
    }
}

async fn handle_client(stream: &mut tokio::net::TcpStream, state: Arc<AppState>) {
    let mut buffer = [0; 1024];
    let mut request = String::new();
    match stream.read(&mut buffer).await {
        Ok(size) => {
            request.push_str(String::from_utf8_lossy(&buffer[..size]).as_ref());
            let (status_line, content) = match &*request {
                r if r.starts_with("OPTIONS") => (CORS_ALLOW_ALL.to_string(), "".to_string()),
                r if r.starts_with("POST /users") => match authenticate(&request).await {
                    Ok(_email) => {
                        let mut client = state
                            .db_pool
                            .get()
                            .await
                            .expect("Failed to get a database connection from the pool");
                        debug!("email {} authenticated", _email);

                        match state.hard_limiter.check() {
                            Ok(()) => create_user::handle(r, &mut client).await,
                            Err(_) => (
                                TOO_MANY_REQUEST.to_string(),
                                "Too Many Requests".to_string(),
                            ),
                        }
                    }
                    Err(_) => {
                        error!("Unauthorized access");
                        (UNAUTHORIZED.to_string(), "Unauthorized".to_string())
                    }
                },
                r if r.starts_with("GET /ping") => (
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n".to_string(),
                    "{\"message\": \"pong\"}".to_string(),
                ),
                r if r.starts_with("GET /users/") => {
                    let client = state
                        .db_pool
                        .get()
                        .await
                        .expect("Failed to get a database connection from the pool");
                    get_user::handle(r, &client).await
                }
                r if r.starts_with("GET /users") => {
                    let client = state
                        .db_pool
                        .get()
                        .await
                        .expect("Failed to get a database connection from the pool");
                    list_user::handle(r, &client).await
                }
                r if r.starts_with("PUT /users/") => match state.common_limiter.check() {
                    Ok(()) => {
                        let client = state
                            .db_pool
                            .get()
                            .await
                            .expect("Failed to get a database connection from the pool");
                        edit_user::handle(r, &client).await
                    }
                    Err(_) => {
                        error!("429 Too Many Requests");
                        (NOT_FOUND.to_string(), "429 Too Many Requests".to_string())
                    }
                },
                r if r.starts_with("DELETE /users/") => match state.common_limiter.check() {
                    Ok(()) => {
                        let client = state
                            .db_pool
                            .get()
                            .await
                            .expect("Failed to get a database connection from the pool");
                        delete_user::handle(r, &client).await
                    }
                    Err(_) => {
                        error!("429 Too Many Requests");
                        (NOT_FOUND.to_string(), "429 Too Many Requests".to_string())
                    }
                },
                r if r.starts_with("POST /login") => match state.hard_limiter.check() {
                    Ok(()) => {
                        let client = state
                            .db_pool
                            .get()
                            .await
                            .expect("Failed to get a database connection from the pool");
                        login_user::handle(r, &client).await
                    }
                    Err(_) => {
                        error!("429 Too Many Requests");
                        (NOT_FOUND.to_string(), "429 Too Many Requests".to_string())
                    }
                },
                _ => (NOT_FOUND.to_string(), "404 not found".to_string()),
            };
            stream
                .write_all(format!("{}{}", status_line, content).as_bytes())
                .await
                .expect("Failed to write response to stream");
        }
        Err(e) => eprintln!("Unable to read stream: {}", e),
    }
}

async fn shutdown_signal_listener() -> tokio::sync::broadcast::Sender<()> {
    let (tx, _) = tokio::sync::broadcast::channel(1);
    let tx_1 = tx.clone();
    tokio::spawn(async move {
        signal::ctrl_c()
            .await
            .expect("Failed to listen for the shutdown signal");
        tx_1.send(()).unwrap();
    });

    tx
}
