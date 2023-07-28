pub mod web {
    use std::io::Read;
    //use std::io::Write;
  	use async_std::stream::StreamExt;   
	use async_std::io::ReadExt;
	use async_std::io::WriteExt;
    
    pub fn urldecode(content: &str) -> Result<String, BacktraceError> {
    	let mut result = std::string::String::new();

		let mut i: usize = 0;
		let mut unicode = std::vec::Vec::<u8>::new(); 
		while i < content.chars().count() {
			let c = content.chars().nth(i).unwrap();
			if c == '%' {
				let code:&str = &content[i + 1..i + 3];
				unicode.push(u8::from_str_radix(code, 16)?);
				i += 3;
			}
            else {
                if unicode.len() > 0 {
				    result += std::str::from_utf8(&unicode)?;
                    unicode.clear(); 
                }

			    if c == '+' {
			    	result.push(' ');
			    	i += 1;
			    }
			    else {
			    	result.push(c);
			    	i += 1;
			    }
            }
		}

        if unicode.len() > 0 {
			result += std::str::from_utf8(&unicode)?;
        }

		Ok(result)
    }

	#[derive(Debug)]
	pub struct BacktraceError {
		err_info: String,
	}

	impl std::fmt::Display for BacktraceError {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			write!(f, "{}", self.err_info)
		}	
	}
	
	impl<E: std::error::Error> From<E> for BacktraceError {
		fn from(err: E) -> BacktraceError {
			let backtrace = format!("Error Description:{}\r\n{}", err.to_string(), std::backtrace::Backtrace::force_capture());
			Self{err_info: backtrace}
		}
	}

    #[derive(PartialEq)]
    #[derive(Debug)]
    #[allow(non_camel_case_types)]
    pub enum JsonType {
        i64(i64),
        f64(f64),
        String(String),
        Vec(Vec<Json>),
        Object(std::collections::HashMap<String, Json>),
        Null,
    }

    impl From<&mut JsonType> for i64 { 
        fn from(item: &mut JsonType) -> Self {
            match item {
                JsonType::i64(val) => *val,
                _ => 0,
            }
        }
    }

    impl From<&mut JsonType> for f64 { 
        fn from(item: &mut JsonType) -> Self {
            match item {
                JsonType::i64(val) => *val as f64,
                JsonType::f64(val) => *val,
                _ => 0.0,
            }
        }
    }

    impl From<&JsonType> for String {
        fn from(item: &JsonType) -> Self {
            match item {
                JsonType::String(val) => val.to_string(),
                JsonType::Null => "null".to_string(),
                _ => "".to_string(),
            }
        }
    }

    impl std::fmt::Display for JsonType {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                JsonType::i64(val) => write!(f, "{:?}", val),
                JsonType::f64(val) => write!(f, "{:?}", val),
                JsonType::String(val) => write!(f, "{:?}", val),
                JsonType::Vec(val) => {
                    write!(f, "[")?;
                    for (count, v) in val.iter().enumerate() {
                        if count != 0 { write!(f, ", ")?; }
                        write!(f, "{}", v)?;
                    }
                    write!(f, "]")
                },
                JsonType::Object(attr) => { 
                    write!(f, "{{")?;
                    for (count, (key, val)) in attr.iter().enumerate() {
                        if count != 0 { write!(f, ", ")?; }
                        write!(f, "{:?}: {}", key, val)?;
                    }
                    write!(f, "}}")
                },
                JsonType::Null => write!(f, "null"),
            }
        }
    }

    impl std::fmt::Display for Json {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            return write!(f, "{}", *self.val);
        }
    }

    #[derive(PartialEq)]
    #[derive(Debug)]
    pub struct Json {
        val: Box<JsonType>,
    }

    impl Json {
        pub fn new(val: JsonType) -> Self {
            Self { 
                val: Box::new(val) 
            }
        }

        pub fn set_val<T: Into<String>>(&mut self, key: T, val: Json) {
            match &mut *self.val {
                JsonType::Object(attr) => {
                    attr.insert(key.into(), val);
                },
                _ => { panic!("not object type");}
            }
        }

        pub fn get_val(&mut self, key: &str) -> &mut JsonType {
            match &mut *self.val {
                JsonType::Object(attr) => {
                    attr.get_mut(key).unwrap().get()
                },
                _ => { panic!("not object type");}
            }
        }

        pub fn get(&mut self) -> &mut JsonType {
            return self.val.as_mut();
        }

		pub fn index(&mut self, index: usize) -> &mut Json {
			if let JsonType::Vec(ref mut vec) = *self.val {
				&mut vec[index]
			}
			else { panic!("not array type");}
		}

		pub fn push(&mut self, val: Json) {
			if let JsonType::Vec(ref mut vec) = *self.val {
				vec.push(val);
			}
			else { panic!("not array type");}
		}

        fn parse_core_number(mut json_iter: &mut (impl Iterator<Item = char> + Clone), cur_json: &mut Json) -> Result<(), &'static str> {
            let mut cache = String::new();
            let mut is_decimal = false;

            let mut copy_iter;
            loop {
                copy_iter = json_iter.clone();
                if let Some(c) = json_iter.next() {
                    match c {
                        '.' => {
                            if is_decimal == true { return Err("double dot");}
                            is_decimal = true; 
                        },
                        _=> {
                            if !c.is_numeric() {break;}
                        }
                    }
                cache.push(c);
                }
                else {
                    break;
                }
            }

            if is_decimal { *cur_json = Json::new(JsonType::f64(cache.parse::<f64>().unwrap())); }
            else { *cur_json = Json::new(JsonType::i64(cache.parse::<i64>().unwrap()));}
            
            *json_iter = copy_iter;
            return Ok(());
        }

        fn parse_core_string(json_iter: &mut (impl Iterator<Item = char> + Clone), cur_json: &mut Json) -> Result<(), &'static str> {
            let mut cache = String::new();
            let mut is_turn = false;//转义

            loop {
                if let Some(c) = json_iter.next() {
                    println!("parse_core_string:[{}]", c);
                    if is_turn {
                        let turn_code: char = match c {
                            'a' => 7 as char,
                            'b' => 8 as char,
                            'f' => 12 as char, 
                            'n' => 10 as char,
                            'r' => 13 as char,
                            't' => 9 as char,
                            'v' => 11 as char,
                            '\\' => 92 as char, 
                            '\'' => 39 as char,
                            '"' => 34 as char,
                            '?' => 64 as char,
                            '0' => 0 as char,
                            _=> return Err("not in turn code map"),
                        };
                        
                        cache.push(turn_code);
                        is_turn = false;
                    }
                    else {
                        match c {
                            '"' => {
                                *cur_json = Json::new(JsonType::String(cache));
                                return Ok(());
                            },
                            '\\' => {
                                is_turn = true;  
                            },
                            _ => {
                                cache.push(c);
                            }
                        }
                    }
                }
                else {
                    break;
                }
            }
            
            Err("not a vaild string")
        }

        fn parse_core_object(json_iter: &mut (impl Iterator<Item = char> + Clone), cur_json: &mut Json) -> Result<(), &'static str> {
            #[derive(PartialEq, Debug)]
            enum ReadState {
                KeyNameStartSign,
                KeyName,
                KeyNameEndSign,
                SplitSign,
                EndSign,
            }

            let mut cache = String::new();
            let mut key_name = String::new();
            let mut cur_state = ReadState::KeyNameStartSign;

            *cur_json = Json::new(JsonType::Object(Default::default()));
            loop {
                if let Some(c) = json_iter.next() { 
                    println!("parse_core_object:[{}], state:[{:?}]", c, cur_state);
                    if c == ' ' { 
                        continue; 
                    }

                    match cur_state {
                        ReadState::KeyNameStartSign => {
                            if c != '\"' { return Err("not key name start sign");}

                            cur_state = ReadState::KeyName;
                        },
                        ReadState::KeyName => {
                            match c {
                                '"' => {
                                    key_name = cache;
                                    cache = String::new();
                                    cur_state = ReadState::SplitSign;
                                },
                                _ => {
                                    cache.push(c);
                                }
                            }
                        },
                        ReadState::KeyNameEndSign => {
                            if c != '\"' { return Err("not key name start sign");}

                            cur_state = ReadState::SplitSign;
                        },
                        ReadState::SplitSign => {
                            if c != ':' { return Err("not key name start sign");}

                            cur_state = ReadState::EndSign;

                            let mut temp = Json::new(JsonType::Null);
                            Self::parse_core(json_iter, &mut temp)?;
                            cur_json.set_val(key_name, temp);
                            key_name = String::new();
                            cur_state = ReadState::EndSign;
                        },
                        ReadState::EndSign => {
                            match c {
                                '}' => {
                                    return Ok(());
                                },
                                ',' => {
                                    cur_state = ReadState::KeyNameStartSign;
                                },
                                _=> {
                                    return Err("not vaild end sign");
                                }
                            }
                        },
                    }
                }
                else {
                    break;
                }
            }
            
            Ok(())
        }

        fn parse_core_array(json_iter: &mut (impl Iterator<Item = char> + Clone), cur_json: &mut Json) -> Result<(), &'static str> {

            *cur_json = Json::new(JsonType::Vec(std::vec::Vec::<Json>::new()));
            loop {
                let mut copy = json_iter.clone();
                if let Some(c) = json_iter.next() {
                    println!("parse_core_array:[{}]", c);
                    match c {
                        ',' => {
                        },
                        ']' => {
                            return Ok(());
                        },
                        _ => {
                            let mut temp =  Json::new(JsonType::Null);
                            Self::parse_core(&mut copy, &mut temp)?;
                            *json_iter = copy;
                            cur_json.push(temp);
                        }
                    }
                }
                else {
                    break; 
                }
            }

            panic!("should not in here");
        }

        fn parse_core_null(json_iter: &mut (impl Iterator<Item = char> + Clone), cur_json: &mut Json) -> Result<(), &'static str> {
            for i in 1..4 {
                if "null".chars().nth(i).unwrap() != json_iter.next().unwrap() { 
                    println!("not null key");
                }
            }
            *cur_json = Json::new(JsonType::Null);
            Ok(())
        }

        fn parse_core(json_iter: &mut (impl Iterator::<Item = char> + Clone), cur_json: &mut Json) -> Result<(), &'static str> {
            loop {
                let mut cur_iter = json_iter.clone();
                if let Some(c) = json_iter.next() {
                    println!("parse_core:[{}]", c);
                    match c {
                        ' ' => {
                        },
                        '{' => {
                            Self::parse_core_object(json_iter, cur_json)?;
                            return Ok(());
                        },
                        '[' => {
                            Self::parse_core_array(json_iter, cur_json)?;
                            return Ok(());
                        },
                        '"' => {
                            if let JsonType::Vec(vec) = cur_json.get() {
                                    let mut temp = Json::new(JsonType::Null);
                                    Self::parse_core_string(json_iter, &mut temp)?;
                                    vec.push(temp);
                            }
                            else {
                                Self::parse_core_string(json_iter, cur_json)?;
                                return Ok(());
                            }
                        },
                        'n' => {
                            Self::parse_core_null(json_iter, cur_json)?;
                            return Ok(());
                        },
                        _ => {
                            if c.is_numeric() {
                                if let JsonType::Vec(vec) = cur_json.get() {
                                    let mut temp = Json::new(JsonType::Null);
                                    Self::parse_core_number(&mut cur_iter, &mut temp)?;
                                    vec.push(temp);
                                }
                                else {
                                    Self::parse_core_number(&mut cur_iter, cur_json)?;
                                    *json_iter = cur_iter;
                                    return Ok(());
                                }
                            }
                            else {
                                println!("parse_core faild:[{}]", c);
                                return Err("not vaild value");
                            }
                        },
                    }
                }
                else {
                    break;
                }
            }

            Ok(())
        }

        pub fn parse(json_str: &str) -> Result<Json, &'static str> {
            let mut result = Json::new(JsonType::Null);
            Self::parse_core(&mut json_str.chars(), &mut result)?;

            //if index < json_str.trim_end().chars().count() { return Err("not vaild json"); }

            Ok(result)
        }
    }

    #[derive(Debug)]
    #[derive(Default)]
    pub struct HttpRequest {
        method: String,
        uri: String,
        version: String,
        header: std::collections::HashMap<String, String>,
        body: std::vec::Vec<u8>,
    }

    impl HttpRequest {
        pub fn new() -> Self{
            return Default::default(); 
        }

        pub fn set_method<T: Into<String>>(&mut self, method: T) {
            self.method = method.into(); 
        }

        pub fn get_method(&self) -> &String {
            &self.method 
        }

        pub fn set_uri<T: Into<String>>(&mut self, uri: T) {
            self.uri = uri.into();
        }

        pub fn get_uri(&self) -> &String {
            &self.uri
        }

        pub fn set_version<T: Into<String>>(&mut self, version: T) {
            self.version = version.into();
        }

        pub fn insert_header<K: Into<String>, V: Into<String>>(&mut self, key: K, val: V) {
            self.header.insert(key.into().to_lowercase(), val.into());
        }

        pub fn get_header(&self, key: &str) -> Option<&String> {
            self.header.get(&key.to_lowercase())
        }

        pub fn set_body(&mut self, buffer: &mut std::vec::Vec<u8>) {
            std::mem::swap(&mut self.body, buffer);
        }

        pub fn get_body(&self) -> &std::vec::Vec<u8> {
           &self.body
        }

        pub fn get_body_len(&self) -> usize {
            if let Some(len_str) = self.get_header("content-length") {
                return str::parse::<usize>(len_str).unwrap()
            }
            else {
                return 0;
            }
        }
    }

    #[derive(Copy, Clone)]
    pub enum HttpResponseStatusCode {
        OK = 200,
        NotFound = 404,
        InternalServerError = 500,
    }

    pub struct HttpResponse {
        version: String, 
        status_code: HttpResponseStatusCode,
        header: std::collections::HashMap<String, String>,  
        body: std::vec::Vec::<u8>,
    }

    impl HttpResponse {
        pub fn new(code: HttpResponseStatusCode,) -> Self {
            Self { 
                version: String::from("HTTP/1.1"), 
                status_code: code,
                header: Default::default(),
                body: Default::default(),
            }
        }

        pub fn view(uri: &str) -> Result<HttpResponse, BacktraceError> {
            const ROOT: &str = "wwwroot";

            let path = format!("{}/{}{}", std::env::current_dir()?.display(), ROOT, uri);
            println!("path:{}", path);

            let file_res = std::fs::OpenOptions::new().read(true).open(path);
            match file_res {
                Ok(mut file) => {
                    let mut buffer = vec![0u8; file.metadata()?.len() as usize];

                    let len = file.read(buffer.as_mut_slice())?;
                    assert_eq!(len, buffer.len());

                    let mut response = HttpResponse::new(HttpResponseStatusCode::OK);
                    response.set_body(buffer);
                    Ok(response)
                },
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        let mut response = HttpResponse::new(HttpResponseStatusCode::NotFound);

                        response.set_body("404 not found".into());

                        Ok(response)
                    }
                    else {
                        Ok(HttpResponse::new(HttpResponseStatusCode::InternalServerError))
                    }
                }
            }
        }

        pub fn json(json_val: Json) -> Self { 
            let data = json_val.to_string().as_bytes().to_vec();

            let mut response = Self {
                version: String::from("HTTP/1.1"), 
                status_code: HttpResponseStatusCode::OK,
                header: Default::default(),
                body: Default::default(), 
            };

            response.set_body(data);
            response.insert_header("content-type", "text/html; charset=utf-8");

            return response;
        }

        pub fn get_status_code(&self) -> HttpResponseStatusCode {
            self.status_code
        }

        pub fn insert_header<K: Into<String>, V: Into<String>>(&mut self, key: K, val: V) {
            self.header.insert(key.into().to_lowercase(), val.into());
        }

        pub fn get_header(&self) -> std::collections::hash_map::Iter<String, String> {
            self.header.iter()
        }

        pub fn set_body(&mut self, body_byte: std::vec::Vec::<u8>) {
            self.body = body_byte;
            self.insert_header("content-length", self.body.len().to_string());
        }

        pub fn get_body(&self) -> &std::vec::Vec::<u8> {
            &self.body
        }

        pub fn get_version(&self) -> &str {
            self.version.as_str()
        } 
    }

    type WebFunc = dyn Fn(Json) -> HttpResponse + Send + Sync;
    type RouterLink = std::collections::HashMap<String, Box<WebFunc>>;
    #[derive(Default)]
    pub struct Router {
        routes: std::collections::HashMap<String, RouterLink>,
    }

    impl Router {
        pub fn new() -> Self {
            Self {
                ..Default::default()
            }
        }

        pub fn register_url<F: Fn(Json) -> HttpResponse + Send + Sync, T: Into<String>>(&mut self, method: T, url: T, func: &'static F) {
            let method: String = method.into();

            if let None = self.routes.get_mut(method.as_str()) {
                self.routes.insert(method.to_string(), RouterLink::new());
            }

            let link = self.routes.get_mut(method.as_str()).unwrap();
            link.insert(url.into(), Box::new(func));
        }

        pub fn contains_url(&self, method: &str, url: &str) -> bool {
            if let Some(link) = self.routes.get(method) {
                return link.contains_key(url);
            }
            else {
                return false;
            }
        }

        pub fn call(&self, method: &str, url: &str, request: &HttpRequest) -> HttpResponse {
            let link = self.routes.get(method).unwrap();
            let func = link.get(url).unwrap();

            let json = Json::parse(std::str::from_utf8(request.get_body()).unwrap()).unwrap();
            func(json)
        }
    }
    
    pub struct HttpServer {
        socket: async_std::net::TcpListener,
        router: std::sync::Arc<Router>,
    }

    impl HttpServer {
        async fn send_response(mut stream: &async_std::net::TcpStream, response: &HttpResponse) -> Result<(), BacktraceError> {
            let head_content = format!
                                (
                                    "{version} {status_code}\r\n{header}\r\n", 
                                    version = response.get_version(), 
                                    status_code = response.get_status_code() as u16,
                                    header = (|| {
                                        let mut result = String::new();
                                        for (key, val) in response.get_header() {
                                            result += format!("{}: {}\r\n", key, val).as_str();
                                        }

                                        return result;
                                    })()
                                );

            let wait_write = vec![head_content.as_bytes(), response.get_body()];
            for cur_buf in wait_write {
                async_std::io::timeout(std::time::Duration::from_millis(5000), stream.write_all(cur_buf)).await?;
            }
            //stream.flush().await.unwrap();

            Ok(())
        }

        fn handle_request(router: &Router, request: &HttpRequest) -> Result<HttpResponse, BacktraceError> {
            let method = request.get_method();
            let uri = request.get_uri();

            println!("handle_request:{}, {}", method, uri);
            if router.contains_url(method, uri) {
                Ok(router.call(method, uri, request))
            }
            else {
                let mut response = HttpResponse::view(uri)?;

                response.insert_header("content-type", "text/html; charset=UTF-8");
                response.insert_header("connection", "keep-alive");

                Ok(response)
            }
        }

        async fn handle_accept(mut stream: &async_std::net::TcpStream) -> Result<HttpRequest, BacktraceError> {
            #[derive(Debug)]
            #[derive(PartialEq)]
            enum HttpState {
                Method,
                URI,
                Version,
                Header,
                Body,
                End,
            }

            let mut cur_state = HttpState::Method;
            let mut cache = std::vec::Vec::<u8>::new();
            let mut http_request = HttpRequest::new();
            
            while cur_state != HttpState::End {
                let mut buf = [0u8; 1024];
                //let buf_size = stream.read(&mut buf).await?;
                let buf_size = async_std::io::timeout(std::time::Duration::from_millis(5000), stream.read(&mut buf)).await?;

                assert!(buf_size > 0);
                //cache += String::from_utf8(buf[0..buf_size].to_vec()).unwrap().as_str();
                cache.append(&mut buf[0..buf_size].to_vec());
                
                //print!("{}", c); 

                while cache.len() > 0 {
                    match cur_state {
                        HttpState::Method => {
                            let content = std::str::from_utf8(cache.as_slice())?;
                            if let Some(pos) = content.find(' ') {
                                http_request.set_method(&content[..pos]);
                                cur_state = HttpState::URI;
                                cache = cache[pos + 1 ..].to_vec();
                            }
                        },
                        HttpState::URI => {
                            let content = std::str::from_utf8(cache.as_slice())?;
                            if let Some(pos) = content.find(' ') {
                                http_request.set_uri(&content[..pos]);
                                cur_state = HttpState::Version;
                                cache = cache[pos + 1 ..].to_vec();
                            }
                        },
                        HttpState::Version => {
                            let content = std::str::from_utf8(cache.as_slice())?;
                            if let Some(pos) = content.find("\r\n") {
                                http_request.set_version(&content[..pos]);
                                cur_state = HttpState::Header;
                                cache = cache[pos + 2 ..].to_vec();
                            }
                        },
                        HttpState::Header => {
                            let content = std::str::from_utf8(cache.as_slice())?;
                            if let Some(pos) = content.find("\r\n") {
                                if pos == 0 {
                                    let body_len = http_request.get_body_len();
                                    if body_len > 0 {
                                        cur_state = HttpState::Body; 
                                    }
                                    else {
                                        cur_state = HttpState::End;
                                    }

                                    cache = cache[2..].to_vec();
                                }
                                else if let Some(key_pos) = content.find(':') {
                                    http_request.insert_header(content[0..key_pos].trim(), content[key_pos + 1..pos].trim());
                                    cache = cache[pos + 2 ..].to_vec();
                                }
                                else {
                                    panic!("header not correct");
                                }
                            } 
                            else {
                                    panic!("header not correct");
                            }
                        },
                        HttpState::Body => {
                            println!("in body:{}", urldecode(std::str::from_utf8(cache.as_slice())?)?);
                            let mut old_body: std::vec::Vec<u8> = http_request.get_body().clone();
                            old_body.append(&mut cache);
                            http_request.set_body(&mut old_body);
                            cur_state = HttpState::End;
                        },
                        HttpState::End => {
                            panic!("should not in here");
                        }
                    }
                }

            }

            
            //println!("{:#?}", http_request);

            //println!("finish!!!!!!!!!!!!");

            Ok(http_request)
        }

        async fn accept_process(router: std::sync::Arc::<Router>, stream: async_std::net::TcpStream) -> Result<(), BacktraceError> {
            println!("accept_process...");
            
            loop {
                let request = Self::handle_accept(&stream).await?;
                let response = Self::handle_request(&router, &request)?;
                Self::send_response(&stream, &response).await?;
            }

            //Ok(())
        }

        pub async fn new(ip_addr: &str) -> Result<Self, BacktraceError> {
            let bind_res = async_std::net::TcpListener::bind(ip_addr).await; 
            match bind_res {
                Ok(val) => { 
                    Ok(
                        Self {
                            socket: val,
                            router: std::sync::Arc::new(Router::new()),
                        }
                    )
                },
                Err(e) => Err(e.into()),
            }
        }

        pub fn get_router(&mut self) -> &mut std::sync::Arc<Router> {
            return &mut self.router;
        }

        pub async fn listen(&self) -> Result<(), BacktraceError> { 
            println!("incoming...");
            println!("{:?}", self.router.routes.get("POST").unwrap().keys());
            while let Some(stream_res) = self.socket.incoming().next().await {			
                let router_copy = std::sync::Arc::clone(&self.router);
                match stream_res {
                   Ok(stream) => {
						let _handle = async_std::task::spawn(async {
                        	if let Err(e) = Self::accept_process(router_copy, stream).await {
								println!("{}", e);
							}
						});
                   },
                   Err(e) => {
                        return Err(e.into());
                   }
                }
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod json_tests {
    #[test]
    fn set_and_get_val() {
        let mut json = crate::web::Json::new(crate::web::JsonType::Object(Default::default()));

        json.set_val("asd", crate::web::Json::new(crate::web::JsonType::i64(123)));

        assert_eq!(i64::from(json.get_val("asd")), 123);
    }

    #[test]
    #[should_panic]
    fn get_null_val() {
        let mut json = crate::web::Json::new(crate::web::JsonType::Null);

        json.get_val("asd"); 
    }

    #[test]
    fn parse_json() {
        let json_str = " { \"a\": 123, \"fgfgfg\": 444.2, \"complex\": { \"son\": 123}, \"c\": \"aaad\" } ";
        let mut json = crate::web::Json::parse(json_str).unwrap();

        println!("test_display:\n{}", json);

        assert_eq!(i64::from(json.get_val("a")), 123);
    }

    #[test]
    fn null_json() {
        let json_str = "null";

        let mut json = crate::web::Json::parse(json_str).unwrap();
        
        println!("test_display:\n{}", json);
        assert_eq!("null", String::from(&*json.get()));
    }

	#[test]
	fn array_json_number() {
		let json_str = "[  703,26,322]";

		let mut json = crate::web::Json::parse(json_str).unwrap();

		assert_eq!(crate::web::JsonType::i64(703), *json.index(0).get());
		assert_eq!(crate::web::JsonType::i64(26), *json.index(1).get());
		assert_eq!(crate::web::JsonType::i64(322), *json.index(2).get());
	}

	#[test]
	fn array_json_decimal() {
		let json_str = "[7.4,20.3,3.6]";

		let mut json = crate::web::Json::parse(json_str).unwrap();

		assert_eq!(crate::web::JsonType::f64(7.4), *json.index(0).get());
		assert_eq!(crate::web::JsonType::f64(20.3), *json.index(1).get());
		assert_eq!(crate::web::JsonType::f64(3.6), *json.index(2).get());
	}

    #[test]
    fn array_json_string() {
		let json_str = "[\"first\", \"second\", \"第三个\"]";

        let mut json = crate::web::Json::parse(json_str).unwrap();

        println!("{}", json);
		assert_eq!(crate::web::JsonType::String("first".into()), *json.index(0).get());
		assert_eq!(crate::web::JsonType::String("second".into()), *json.index(1).get());
		assert_eq!(crate::web::JsonType::String("第三个".into()), *json.index(2).get());
    }

    #[test]
    fn parse_complex_json_1() {
        let json_str = " { \"a\": 123, \"c\": \"a\\\"ha\\\"aad\", \"fgfgfg\": 444.2, \"complex\": { \"son\": 123}, \"zzz\": null} ";
        let mut json = crate::web::Json::parse(json_str).unwrap();

        println!("test_display:\n{}", json);

        assert_eq!(String::from(&*json.get_val("c")), "a\"ha\"aad");
    }

    #[test]
    fn parse_complex_json_2() {
		let json_str = "[{\"a\": 123, \"bb\": \"abc\"}, {\"a\": 456}, {\"a\": 789}]";

        let mut json = crate::web::Json::parse(json_str).unwrap();

        assert_eq!(crate::web::JsonType::i64(123), *json.index(0).get_val("a"));
        assert_eq!(crate::web::JsonType::String("abc".into()), *json.index(0).get_val("bb"));

        assert_eq!(crate::web::JsonType::i64(456), *json.index(1).get_val("a"));

        assert_eq!(crate::web::JsonType::i64(789), *json.index(2).get_val("a"));
    }

    fn test_iter(iter: &mut (impl Iterator<Item = char> + Clone)) {
        iter.next();
        iter.clone();
    }

    #[test]
    fn parse_complex_json_3() {
        let json_str = "{\"www\": \"中\", \"lalala\": [1, 2, 3]}";

        let mut json = crate::web::Json::parse(json_str).unwrap();
    }
}

#[cfg(test)]
mod urldecode {
	#[test]
	fn test_decode() {
		let code = "name=123&aaa=444&www=%E4%B8%AD%E6%96%87test%F0%9F%92%96";
		
		let result = crate::web::urldecode(code);
		assert_eq!("name=123&aaa=444&www=中文test💖", result.unwrap());
	}
}

#[cfg(test)]
mod router_tests {

    fn test_response(param: crate::web::Json) -> crate::web::HttpResponse {
        println!("test_response!!!param:{}", param);

        return crate::web::HttpResponse::new(crate::web::HttpResponseStatusCode::OK);
    }

    #[test]
    fn call_back_register() {
        let mut router = crate::web::Router::new(); 

        println!("add:{:p}", &test_response);
        router.register_url("GET".to_string(), "asd".to_string(), &test_response);

        let mut request = crate::web::HttpRequest::new();
        request.set_body(&mut "{\"a\": 123123}".as_bytes().to_vec());
        request.insert_header("content-length", request.get_body().len().to_string());

        router.call("GET", "asd", &request);
    }

    #[test]
    fn contains_uri() {
        let mut router = crate::web::Router::new();

        router.register_url("POST", "/asd", &test_response);
        assert_eq!(true, router.contains_url("POST", "/asd"));
        assert_eq!(false, router.contains_url("GET", "/asd"));
        assert_eq!(false, router.contains_url("POST", "asd"));
    }
}

//#[cfg(test)]
//mod server_tests {
//    use route_macro_attribute::route;
//
//    #[route(GET, "/")]
//    #[test]
//    fn index() {
//    }
//     
//    #[test]
//    fn listen() {
//        let server_res = async_std::task::block_on(crate::web::HttpServer::new("0.0.0.0:9999"));
//        match server_res {
//            Ok(mut server) => {  
//                if let Err(e) = async_std::task::block_on(server.listen()) {
//                    panic!("{}", e);
//                }
//            },
//            Err(err_info) => {
//                panic!("{}", err_info);
//            }
//        }
//    }
//}
