mod users;
mod auth;
mod libs;

use std::sync::Arc;
use std::num::NonZeroU32;
use std::str::FromStr; 
use std::io::Write; 
use governor::clock::QuantaClock;
use governor::state::{InMemoryState, NotKeyed};
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use tokio_postgres::{Config, NoTls};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{Duration, sleep};
use tokio::signal;
use governor::{Quota, RateLimiter};
use deadpool_postgres::{Manager, Pool};
use log::{info, error, debug};
use users::handler::{ create_user, get_user, list_user, edit_user, delete_user };
use auth::handler::login_user;
use libs::{ get_db_url, authenticate, NOT_FOUND, CORS_ALLOW_ALL, TOO_MANY_REQUEST, UNAUTHORIZED };


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
    let pool = Pool::new(manager, 16); // 16 adalah ukuran maksimum pool

    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)  
        .format(|buf: &mut env_logger::fmt::Formatter, record| {
            writeln!(buf, "[{}] {}:{} - {}: {}", 
                 buf.timestamp(),
                 record.file().unwrap_or("<unknown>"),
                 record.line().unwrap_or(0),
                 record.level(),
                 record.args())
        })
        .init();

    //start server and print port
    let listener = TcpListener::bind("0.0.0.0:8080").await.expect("Failed to bind to address 0.0.0.0:8080");
    let semaphore = Arc::new(Semaphore::new(10));
    let global_limiter = Arc::new(RateLimiter::direct(Quota::per_second(NonZeroU32::new(200).unwrap())));
    let common_limiter = Arc::new(RateLimiter::direct(Quota::per_second(NonZeroU32::new(100).unwrap())));
    let hard_limiter = Arc::new(RateLimiter::direct(Quota::per_second(NonZeroU32::new(100).unwrap())));
    info!("Server listening on port 8080");

    // Share AppState with all incoming connections
    let app_state = Arc::new(AppState {
        db_pool: pool,
        common_limiter: common_limiter.clone(),
        hard_limiter: hard_limiter.clone(),
    });
    
    let shutdown_signal = signal::ctrl_c(); 

    let server_task = tokio::spawn(async move {
        loop {
            let (mut stream, _) = listener.accept().await.expect("Failed to accept connection");
            let permit = semaphore.clone().acquire_owned().await.expect("Failed to acquire semaphore permit"); 
            let state = app_state.clone();
            let global_limiter = global_limiter.clone();
                
            tokio::spawn(async move {
                loop {
                    match global_limiter.check() {
                        Ok(()) => {
                            handle_client(&mut stream, state).await; 
                            break;
                        },
                        Err(_) => sleep(Duration::from_millis(100)).await
                    }                
                }
                drop(permit);
            });
        }
    });
    let _ = shutdown_signal.await;
    println!("Shutting down gracefully...");
    server_task.abort(); 
}

async fn handle_client(stream: &mut tokio::net::TcpStream, state: Arc<AppState>) {
    let mut buffer = [0; 1024];
    let mut request = String::new();
    match stream.read(&mut buffer).await {
        Ok(size) => {
            request.push_str(String::from_utf8_lossy(&buffer[..size]).as_ref());
            let mut client = state.db_pool.get().await.expect("Failed to get a database connection from the pool");
            let (status_line, content) = match &*request {
                r if r.starts_with("OPTIONS") => (CORS_ALLOW_ALL.to_string(),"".to_string()),
                r if r.starts_with("POST /users") => {
                    match authenticate(&request).await {
                        Ok(_email) => {
                            debug!("email {} authenticated", _email);
                            match state.hard_limiter.check() {
                                Ok(()) => create_user::handle(r, &mut client).await,
                                Err(_) => (TOO_MANY_REQUEST.to_string(), "Too Many Requests".to_string())
                            }
                        }
                        Err(_) => {
                            error!("Unauthorized access");
                            (UNAUTHORIZED.to_string(), "Unauthorized".to_string())
                        }
                    }
                },
                r if r.starts_with("GET /users/") => get_user::handle(r, &client).await,
                r if r.starts_with("GET /users") => list_user::handle(r, &client).await,
                r if r.starts_with("PUT /users/") => {
                    match state.common_limiter.check() {
                        Ok(()) => edit_user::handle(r, &client).await,
                        Err(_) => {
                            error!("429 Too Many Requests");
                            (NOT_FOUND.to_string(), "429 Too Many Requests".to_string())
                        }
                        
                    }
                },
                r if r.starts_with("DELETE /users/") => {
                    match state.common_limiter.check() {
                        Ok(()) => delete_user::handle(r, &client).await,
                        Err(_) => {
                            error!("429 Too Many Requests");
                            (NOT_FOUND.to_string(), "429 Too Many Requests".to_string())
                        }
                    }
                },
                r if r.starts_with("POST /login") => {
                    match state.hard_limiter.check() {
                        Ok(()) => login_user::handle(r, &client).await,
                        Err(_) => {
                            error!("429 Too Many Requests");
                            (NOT_FOUND.to_string(), "429 Too Many Requests".to_string())
                        }   
                    }
                },
                _ => (NOT_FOUND.to_string(), "404 not found".to_string()),
            };
            stream.write_all(format!("{}{}", status_line, content).as_bytes()).await.expect("Failed to write response to stream");
        }
        Err(e) => eprintln!("Unable to read stream: {}", e),
    }
}