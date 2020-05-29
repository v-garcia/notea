use warp::{Filter};

// static DB_PATH: &str = "target/my_store";

#[tokio::main]
async fn main() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"

    let api = warp::path("api");

    let post_file = warp::post()
                        .and(warp::path::param::<String>())
                        .and(warp::path("set"))
                        .map(| area | format!("Hello, World! {}", area));
                        
    let routes = warp::any().and(api).and(post_file).and(warp::path::end());



    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;

        
}

