pub mod web {
    use std::io::Read;

    #[derive(PartialEq)]
    #[derive(Debug)]
    #[allow(non_camel_case_types)]
    pub enum JsonType {
        i64(i64),
        f64(f64),
        String(String),
        Vec(Vec<JsonType>),
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
                        write!(f, "{:?}", v)?;
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

        pub fn parse(json_str: &str) -> Result<Json, String> {
            let mut result = std::collections::VecDeque::<Json>::new();
            result.push_back(Json::new(JsonType::Null));

            #[derive(PartialEq)]
            enum ReadState {
                KeyNameSignal,
                KeyName,
                Val,
                ValNumber,
                ValNumberDecial,
                ValString,
                ValStringTurnCode,//转义字符处理
                WaitSignal(char), 
                EndSignal,
            }

            let mut cur_state = ReadState::Val;
            let mut cache = String::new();
            let mut key_name_stack = std::collections::VecDeque::<String>::new();
            let mut val_str: String;
            let mut index = 0;

            while index < json_str.len() {
                let c: char = json_str.chars().nth(index).unwrap();
            //for c in json_str.chars() {
                if c == ' ' { 
                    index = index + 1;
                    continue; 
                }

                //println!("loop [{}]", c);

                match cur_state {
                    ReadState::KeyNameSignal => {
                        if c != '"' { return Err("not key name signal".to_string()) };

                        cur_state = ReadState::KeyName;
                        index = index + 1;
                    },
                    ReadState::KeyName => {
                        match c {
                            '"' => {
                                cur_state = ReadState::WaitSignal(':');
                                key_name_stack.push_back(cache);
                                //println!("key:[{}]", &key_name_stack.back().unwrap());
                                result.push_back(Json::new(JsonType::Null));
                                cache = String::new();
                            },
                            _ => {
                                cache.push(c); 
                            }
                        }
                        index = index + 1;
                    },
                    ReadState::Val => {
                        match c {
                            '{' => {
                                cur_state = ReadState::KeyNameSignal;
                                *result.back_mut().unwrap() = Json::new(JsonType::Object(Default::default()));
                                index = index + 1;
                            },
                            '"' => {
                                cur_state = ReadState::ValString; 
                                index = index + 1;
                            }, 
                            'n' => {
                                if &json_str[index..index + 4] != "null" { return Err("undefined value".to_string()); }

                                *result.back_mut().unwrap() = Json::new(JsonType::Null);
                                index = index + 4;

                                cur_state = ReadState::EndSignal;
                            }
                            _ => {
                                if c.is_numeric() { 
                                    cur_state = ReadState::ValNumber; 
                                    cache.push(c);
                                    index = index + 1;
                                }
                                else { return Err("??? wtf".to_string()); }
                            },
                        }
                    },
                    ReadState::ValNumber => {
                        match c {
                            ',' | '}' => {
                                cur_state = ReadState::EndSignal;

                                val_str = cache;
                                cache = String::new();

                                *result.back_mut().unwrap() = Json::new(JsonType::i64(val_str.parse::<i64>().unwrap()));
                            },
                            '.' => {
                                cur_state = ReadState::ValNumberDecial;
                                index = index + 1;
                            },
                            _ => {
                                if !c.is_numeric() { return Err("not number".to_string());}

                                cache.push(c); 

                                index = index + 1;
                            },
                        }
                    },
                    ReadState::ValNumberDecial => {
                        match c {
                            ',' | '}' => {
                                cur_state = ReadState::EndSignal;

                                val_str = cache;
                                cache = String::new();

                                *result.back_mut().unwrap() = Json::new(JsonType::f64(val_str.parse::<f64>().unwrap()));
                            },
                            _ => {
                                if !c.is_numeric() && c != '.' { return Err("not number".to_string());}

                                cache.push(c); 
                                index = index + 1;
                            },
                        }
                    },
                    ReadState::ValString => {
                        match c {
                            '"' => { 
                                val_str = cache;
                                cache = String::new();

                                *result.back_mut().unwrap() = Json::new(JsonType::String(val_str));

                                cur_state = ReadState::EndSignal;
                            },
                            '\\' => {
                                cur_state = ReadState::ValStringTurnCode;
                            },
                            _ => { 
                                cache.push(c);
                            },
                        }

                        index = index + 1;
                    },
                    ReadState::ValStringTurnCode => {
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
                            _=> return Err(format!("{} not in turn code map", c)),
                        };

                        cache.push(turn_code);
                        index = index + 1;
                        cur_state = ReadState::ValString;
                    },
                    ReadState::WaitSignal(val) => {
                        if c != val { return Err("wait signal not correct".to_string()); }

                        match c {
                            ':' => {
                                cur_state = ReadState::Val;
                            },
                            _ => {
                            }
                        }

                        index = index + 1;
                    },
                    ReadState::EndSignal => {
                        match c {
                            '}' => {},
                            ',' => { cur_state = ReadState::KeyNameSignal; },
                            _ => {
                                return Err("not end signal".to_string()); 
                            },
                        }

                        let temp = result.pop_back().unwrap();
                        result.back_mut().unwrap().set_val(key_name_stack.pop_back().unwrap(), temp);
                        index = index + 1;
                    }
                }
            }

            //println!("stack:{}", result.len());

            return Ok(result.pop_back().unwrap());
        }
    }

    #[derive(Debug)]
    #[derive(Default)]
    pub struct HttpRequest {
        method: String,
        uri: String,
        version: String,
        header: std::collections::HashMap<String, String>,
    }

    impl HttpRequest {
        pub fn new() -> Self{
            return Default::default(); 
        }

        pub fn set_method<T: Into<String>>(&mut self, method: T) {
            self.method = method.into(); 
        }

        pub fn set_uri<T: Into<String>>(&mut self, uri: T) {
            self.uri = uri.into();
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

        pub fn get_body_len(&self) -> Result<usize, <usize as std::str::FromStr>::Err> {
            if let Some(body_len_str) = self.get_header("content-length") {
                let body_len = body_len_str.parse::<usize>()?;
                Ok(body_len)
            }
            else {
                Ok(0)
            }
        }
    }

    pub struct HttpServer {
        socket: std::net::TcpListener,
    }

    impl HttpServer {
        fn handle_accept(mut stream: std::net::TcpStream) -> Result<HttpRequest, Box<dyn std::error::Error>> {
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
            let mut cache = String::new();
            let mut http_request = HttpRequest::new();
            
            while cur_state != HttpState::End {
                let mut buf = [0u8; 1024];
                let buf_size = stream.read(&mut buf)?;

                if buf_size != 0 {
                    cache += String::from_utf8(buf[0..buf_size].to_vec()).unwrap().as_str();
                }
                else {
                    break;
                }
                
                //print!("{}", c); 

                while cache.len() > 0 {
                    println!("cur_state:{:#?}", cur_state);
                    println!("cache_size:[{}]", cache.len());
                    for c in cache.chars() {
                        println!("cache:[{}] as u8: [{}]", c, c as u8);
                    }
                    
                    match cur_state {
                        HttpState::Method => {
                            if let Some(pos) = cache.find(' ') {
                                http_request.set_method(&cache[..pos]);
                                cur_state = HttpState::URI;
                                cache = cache[pos + 1 ..].to_string();
                            }
                        },
                        HttpState::URI => {
                            if let Some(pos) = cache.find(' ') {
                                http_request.set_uri(&cache[..pos]);
                                cur_state = HttpState::Version;
                                cache = cache[pos + 1 ..].to_string();
                            }
                        },
                        HttpState::Version => {
                            if let Some(pos) = cache.find("\r\n") {
                                http_request.set_version(&cache[..pos]);
                                cur_state = HttpState::Header;
                                cache = cache[pos + 2 ..].to_string();
                            }
                        },
                        HttpState::Header => {
                            if let Some(pos) = cache.find("\r\n") {
                                if pos == 0 {
                                    let body_len = http_request.get_body_len()?;
                                    if body_len > 0 {
                                        cur_state = HttpState::Body; 
                                    }
                                    else {
                                        cur_state = HttpState::End;
                                    }

                                    cache = cache[2..].to_string();
                                }
                                else if let Some(key_pos) = cache.find(':') {
                                    http_request.insert_header(cache[0..key_pos].trim(), cache[key_pos + 1..pos].trim());
                                    cache = cache[pos + 2 ..].to_string();
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
                            println!("in body");
                            cache.clear();
                            break;
                        },
                        HttpState::End => {
                            panic!("should not in here");
                        }
                    }
                }

                println!("read({}):", buf_size);
            }

            
            println!("{:#?}", http_request);

            println!("finish!!!!!!!!!!!!");

            Ok(http_request)
        }

        pub fn new(ip_addr: &str) -> Result<Self, std::io::Error> {
            let bind_res = std::net::TcpListener::bind(ip_addr); 
            match bind_res {
                Ok(val) => { 
                    Ok(
                        Self {
                            socket: val,
                        }
                    )
                },
                Err(e) => Err(e),
            }
        }

        pub fn listen(&self) -> Result<&Self, Box<dyn std::error::Error>> {
            println!("incoming...");
            for stream_res in self.socket.incoming() {
                match stream_res {
                   Ok(stream) => {
                       Self::handle_accept(stream)?;
                   },
                   Err(e) => {
                        return Err(Box::new(e)) 
                   }
                }
            }

            Ok(self)
        }
    }
}

#[cfg(test)]
mod web_tests {
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
    fn parse_complex_json() {
        let json_str = " { \"a\": 123, \"c\": \"a\\\"ha\\\"aad\", \"fgfgfg\": 444.2, \"complex\": { \"son\": 123}, \"zzz\": null} ";
        println!("the json:\n{}", json_str);
        println!("laala:'{}'", "\"");
        let mut json = crate::web::Json::parse(json_str).unwrap();

        println!("test_display:\n{}", json);

        assert_eq!(String::from(&*json.get_val("c")), "a\"ha\"aad");
    }

    #[test]
    fn null_json() {
        let json_str = "null";

        let mut json = crate::web::Json::parse(json_str).unwrap();
        
        println!("test_display:\n{}", json);
        assert_eq!("null", String::from(&*json.get()));
    }
}

#[cfg(test)]
mod server_tests {
     
    #[test]
    fn listen() {
        let server_res = crate::web::HttpServer::new("0.0.0.0:9999");
        match server_res {
            Ok(server) => {
                if let Err(e) = server.listen() {
                    panic!("{}", e);
                }
            },
            Err(err_info) => {
                panic!("{}", err_info);
            }
        }
    }
}
