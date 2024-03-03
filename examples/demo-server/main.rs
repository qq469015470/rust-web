fn home(_param: rust_web::web::Json) -> rust_web::web::HttpResponse {
    return rust_web::web::HttpResponse::view("index.html").unwrap();
}

fn test_response(param: rust_web::web::Json) -> rust_web::web::HttpResponse {
    println!("test_response!!!param:{}", param);

    std::thread::sleep(std::time::Duration::from_millis(5000));

    return rust_web::web::HttpResponse::json(param);
}

fn main() {
    let server_res = async_std::task::block_on(rust_web::web::HttpServer::new("0.0.0.0:9002"));
    match server_res {
        Ok(mut server) => {  
            server.use_ssl(true);
            let mut router_arc = server.get_router();

            let router = std::sync::Arc::get_mut(&mut router_arc).unwrap();

            router.register_url("GET", "/", &home);
            router.register_url("POST", "/test", &test_response);

            if let Err(e) = async_std::task::block_on(server.listen()) {
                panic!("{}", e);
            }
        },
        Err(err_info) => {
            panic!("{}", err_info);
        }
    }
}
