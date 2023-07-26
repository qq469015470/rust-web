fn test_response(param: rust_web::web::Json) -> rust_web::web::HttpResponse {
    println!("test_response!!!param:{}", param);

    let mut response = rust_web::web::HttpResponse::new(rust_web::web::HttpResponseStatusCode::OK);

    response.insert_header("content-length", "0");

    return response;
}

fn main() {
    let server_res = async_std::task::block_on(rust_web::web::HttpServer::new("0.0.0.0:9999"));
    match server_res {
        Ok(mut server) => {  
            let mut router_arc = server.get_router();

            let router = std::sync::Arc::get_mut(&mut router_arc).unwrap();

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
