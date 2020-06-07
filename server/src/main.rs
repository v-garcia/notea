use notea::{Store, StoreUserError};
use std::convert::Infallible;
use std::sync::Arc;
use warp::{http, Buf, Filter};

// static DB_PATH: &str = "target/my_store";

const WEB_ROOT: &'static str = "./www/static";

fn with_store(
    store: Arc<Store>,
) -> impl Filter<Extract = (Arc<Store>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || store.clone())
}

pub struct PostData {
    pub key: String,
    pub hash: Option<String>,
    pub new_hash: String,
    pub data: bytes::Bytes,
}

pub async fn insert_data(
    post_data: PostData,
    store: Arc<Store>,
) -> Result<impl warp::Reply, Infallible> {
    let stored = store.set(
        &post_data.key,
        post_data.data.bytes(),
        post_data.hash.as_deref(),
        &post_data.new_hash,
    );

    let r: Box<dyn warp::Reply> = match stored {
        Ok(_) => Box::new(http::StatusCode::OK),
        Err(e) => Box::new(warp::reply::with_status(
            e.to_string(),
            http::StatusCode::BAD_REQUEST,
        )),
    };

    return Ok(r);
}

pub async fn get_data(key: String, store: Arc<Store>) -> Result<impl warp::Reply, Infallible> {
    let stored = store.get(&key);

    let r: Box<dyn warp::Reply> = match stored {
        Ok(None) => Box::new(http::StatusCode::NOT_FOUND),
        Ok(Some(e)) => Box::new(warp::reply::with_header(
            e,
            "Content-Type",
            "application/octet-stream",
        )),
        Err(e) => Box::new(warp::reply::with_status(
            e.to_string(),
            http::StatusCode::INTERNAL_SERVER_ERROR,
        )),
    };

    return Ok(r);
}

#[tokio::main]
async fn main() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"

    let store = Arc::new(Store::init_from_path("target/my-store").unwrap());

    let post_file = warp::post()
        .and(warp::path("data"))
        .and(warp::path::param::<String>())
        .and(warp::body::content_length_limit(1024 * 1024 * 100)) // 100 Mb max
        .and(warp::header::exact(
            "Content-Type",
            "application/octet-stream",
        ))
        .and(warp::header::optional("Content-Hash"))
        .and(warp::header::header("New-Content-Hash"))
        .and(warp::body::bytes())
        .map(
            move |key: String,
                  hash: Option<String>,
                  new_hash: String,
                  bytes: bytes::Bytes|
                  -> PostData {
                PostData {
                    key: key,
                    hash: hash,
                    new_hash: new_hash,
                    data: bytes,
                }
            },
        )
        .and(with_store(store.clone()))
        .and_then(insert_data);

    let get_file = warp::get()
        .and(warp::path("data"))
        .and(warp::path::param::<String>())
        .and(with_store(store.clone()))
        .and_then(get_data);

    let api = warp::path("api").and(post_file.or(get_file));

    let static_content = warp::path("static").and(warp::fs::dir(format!("{}/assets", WEB_ROOT)));

    let main = warp::path::end().and(warp::filters::fs::file(format!(
        "{}/new-pad-wizard.html",
        WEB_ROOT
    )));

    let pad = warp::get().and(warp::filters::fs::file(format!("{}/pad.html", WEB_ROOT)));

    let routes = warp::any().and(api.or(static_content).or(main).or(pad));

    // warp::serve(warp::path("static").and(warp::fs::dir(format!("{}/assets", WEB_ROOT))))
    //     .run(([127, 0, 0, 1], 3030))
    //     .await;
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
