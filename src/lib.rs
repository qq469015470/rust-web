pub mod web {
    use std::io::Read;
    use std::io::Write;
    use num_enum::TryFromPrimitive;
  	//use async_std::stream::StreamExt;   
	//use async_std::io::ReadExt;
	//use async_std::io::WriteExt;
    
    pub fn urldecode<T: AsRef<str>>(content: T) -> Result<String, BacktraceError> {
    	let mut result = std::string::String::new();

		let mut i: usize = 0;
		let mut unicode = std::vec::Vec::<u8>::new(); 
		while i < content.as_ref().chars().count() {
			let c = content.as_ref().chars().nth(i).unwrap();
			if c == '%' {
				let code:&str = &content.as_ref()[i + 1..i + 3];
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
        err_desc: String,
		err_info: String,
	}

	impl std::fmt::Display for BacktraceError {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			write!(f, "Error Description:{}\r\n{}", self.err_desc, self.err_info)
		}	
	}
	
	impl<E: std::error::Error + Sized> From<E> for BacktraceError {
		fn from(err: E) -> BacktraceError {
			Self { 
                err_desc: err.to_string(),
                err_info: std::backtrace::Backtrace::force_capture().to_string(),
            }
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

    impl From<&JsonType> for i64 { 
        fn from(item: &JsonType) -> Self {
            match item {
                JsonType::i64(val) => *val,
                JsonType::f64(val) => *val as i64,
                _ => panic!("can not parse"),
            }
        }
    }

    impl From<&mut JsonType> for i64 { 
        fn from(item: &mut JsonType) -> Self {
            let item: &JsonType = item;
            Self::from(item)
        }
    }

    impl From<&JsonType> for f64 { 
        fn from(item: &JsonType) -> Self {
            match item {
                JsonType::i64(val) => *val as f64,
                JsonType::f64(val) => *val,
                JsonType::String(val) => val.parse::<f64>().unwrap(),
                _ => panic!("can not parse"),
            }
        }
    }

    impl From<&mut JsonType> for f64 { 
        fn from(item: &mut JsonType) -> Self {
            let item:&JsonType = item;
            Self::from(item)
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

    impl From<&mut JsonType> for String {
        fn from(item: &mut JsonType) -> Self {
            let item: &JsonType = item;
            Self::from(item)
        }
    }

    impl<const N: usize> From<[(String, Json); N]> for JsonType {
        fn from(item: [(String, Json); N]) -> Self {
            JsonType::Object(std::collections::HashMap::from(item))
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

    impl From<&str> for Json {
        fn from(item: &str) -> Self {
            item.to_string().into()
        }
    }

    impl From<String> for Json {
        fn from(item: String) -> Self {
            Json::new(JsonType::String(item))
        }
    }

    impl From<f64> for Json {
        fn from(item: f64) -> Self {
            Json::new(JsonType::f64(item))
        }
    }

    impl From<i64> for Json {
        fn from(item: i64) -> Self {
            Json::new(JsonType::i64(item))
        }
    }

    //impl From<std::collections::HashMap<String, Json>> for Json {
    //    fn from(item: std::collections::HashMap<String, Json>) -> Self {
    //        Json::new(JsonType::Object(item))
    //    }
    //}

    impl<const N: usize> From<[(String, Json); N]> for Json {
        fn from(item: [(String, Json); N]) -> Self {
            Json::new(JsonType::Object(item.into()))
        }
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

        pub fn get_val(&self, key: &str) -> Option<&JsonType> {
            match &*self.val {
                JsonType::Object(attr) => {
                    let Some(son_obj) = attr.get(key) else { return None; };
                    Some(son_obj.get())
                },
                _ => { None }
            }
        }

        pub fn get_val_mut(&mut self, key: &str) -> Option<&mut JsonType> {
            match &mut *self.val {
                JsonType::Object(attr) => {
                    let Some(son_obj) = attr.get_mut(key) else { return None; };
                    Some(son_obj.get_mut())
                },
                _ => { None }
            }
        }

        pub fn get(&self) -> &JsonType {
            return self.val.as_ref();
        }

        pub fn get_mut(&mut self) -> &mut JsonType {
            return self.val.as_mut();
        }

		pub fn index(&self, index: usize) -> Option<&Json> {
			if let JsonType::Vec(ref vec) = *self.val {
				Some(&vec[index])
			}
			else { None }
		}

		pub fn push(&mut self, val: Json) {
			if let JsonType::Vec(ref mut vec) = *self.val {
				vec.push(val);
			}
			else { panic!("not array type");}
		}

        fn parse_core_number(json_iter: &mut (impl Iterator<Item = char> + Clone), cur_json: &mut Json) -> Result<(), BacktraceError> {
            let mut cache = String::new();
            let mut is_decimal = false;

            let mut copy_iter;
            loop {
                copy_iter = json_iter.clone();
                if let Some(c) = json_iter.next() {
                    match c {
                        '.' => {
                            if is_decimal == true { return Err(std::io::Error::new(std::io::ErrorKind::Other, "double dot").into());}
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

        fn parse_core_string(json_iter: &mut (impl Iterator<Item = char> + Clone), cur_json: &mut Json) -> Result<(), BacktraceError> {
            let mut cache = String::new();
            let mut is_turn = false;//转义

            loop {
                if let Some(c) = json_iter.next() {
                    //println!("parse_core_string:[{}]", c);
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
                            _=> return Err(std::io::Error::new(std::io::ErrorKind::Other, "not in turn code map").into()),
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
            
            Err(std::io::Error::new(std::io::ErrorKind::Other, "not a vaild string").into())
        }

        fn parse_core_object(json_iter: &mut (impl Iterator<Item = char> + Clone), cur_json: &mut Json) -> Result<(), BacktraceError> {
            #[derive(PartialEq, Debug)]
            enum ReadState {
                KeyNameStartSign,
                KeyName,
                SplitSign,
                EndSign,
            }

            let mut cache = String::new();
            let mut key_name = String::new();
            let mut cur_state = ReadState::KeyNameStartSign;

            *cur_json = Json::new(JsonType::Object(Default::default()));
            loop {
                if let Some(c) = json_iter.next() { 
                    //println!("parse_core_object:[{}], state:[{:?}]", c, cur_state);
                    if c == ' ' || c == '\n' || c == '\t' { 
                        continue; 
                    }

                    match cur_state {
                        ReadState::KeyNameStartSign => {
                            if c != '\"' { return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("'{}' not key name start sign", c)).into());}

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
                        ReadState::SplitSign => {
                            if c != ':' { return Err(std::io::Error::new(std::io::ErrorKind::Other, "not key name start sign").into());}

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
                                    return Err(std::io::Error::new(std::io::ErrorKind::Other, "not vaild end sign").into());
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

        fn parse_core_array(json_iter: &mut (impl Iterator<Item = char> + Clone), cur_json: &mut Json) -> Result<(), BacktraceError> {

            *cur_json = Json::new(JsonType::Vec(std::vec::Vec::<Json>::new()));
            loop {
                let mut copy = json_iter.clone();
                if let Some(c) = json_iter.next() {
                    //println!("parse_core_array:[{}]", c);
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

        fn parse_core_null(json_iter: &mut (impl Iterator<Item = char> + Clone), cur_json: &mut Json) -> Result<(), BacktraceError> {
            for i in 1..4 {
                if "null".chars().nth(i).unwrap() != json_iter.next().unwrap() { 
                    //println!("not null key");
                }
            }
            *cur_json = Json::new(JsonType::Null);
            Ok(())
        }

        fn parse_core(json_iter: &mut (impl Iterator::<Item = char> + Clone), cur_json: &mut Json) -> Result<(), BacktraceError> {
            loop {
                let mut cur_iter = json_iter.clone();
                if let Some(c) = json_iter.next() {
                    //println!("parse_core:[{}]", c);
                    match c {
                        ' ' => {}, '\n' => {}, '\t' => {},
                        '{' => {
                            Self::parse_core_object(json_iter, cur_json)?;
                            return Ok(());
                        },
                        '[' => {
                            Self::parse_core_array(json_iter, cur_json)?;
                            return Ok(());
                        },
                        '"' => {
                            if let JsonType::Vec(vec) = cur_json.get_mut() {
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
                                if let JsonType::Vec(vec) = cur_json.get_mut() {
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
                                //println!("parse_core faild:[{}]", c);
                                return Err(std::io::Error::new(std::io::ErrorKind::Other, "not vaild value").into());
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

        pub fn parse<T: AsRef<str>>(json_str: T) -> Result<Json, BacktraceError> {
            let mut result = Json::new(JsonType::Null);
            Self::parse_core(&mut json_str.as_ref().chars(), &mut result)?;

            //if index < json_str.trim_end().chars().count() { return Err("not vaild json"); }

            Ok(result)
        }

        pub fn parse_form_data<T: AsRef<str>>(form_data: T) -> Result<Json, BacktraceError> {
            let form_data = form_data.as_ref();
            let mut left = 0;
            let mut right_opt = form_data.find("=");

            let mut json_obj = Json::new(JsonType::Object(Default::default()));
            while let Some(right) = right_opt {
                let key = &form_data[left..][..right];
                //println!("key:{key}");

                let mut key_left = 0;
                let mut key_right_opt = key.find("%5B");

                let mut cur_param: &mut JsonType = if let Some(key_right) = key_right_opt { 

                    if let None = json_obj.get_val_mut(&key[key_left..key_right]) {
                        json_obj.set_val(&key[key_left..key_right], Json::new(JsonType::Null));
                    }
                    
                    json_obj.get_val_mut(&key[key_left..key_right]).unwrap()
                }
                else {
                    if let None = json_obj.get_val_mut(&key[key_left..]) {
                        json_obj.set_val(&key[key_left..], Json::new(JsonType::Null));
                    }

                    json_obj.get_val_mut(&key[key_left..]).unwrap()
                };

                while key_right_opt.is_some() {
                    key_left = key_right_opt.unwrap() + 3;
                    key_right_opt = key[key_left..].find("%5D");

                    let attr = if let Some(key_right) = key_right_opt {
                        key[key_left..][..key_right].to_string()
                    } 
                    else {
                        key[key_left..].to_string()
                    };

                    //println!("attr:\"{attr}\"");
                    if attr != "" {
                        //let JsonType::Object(ref mut temp) = cur_param else { return Err("must be obj".into());};
                        match cur_param {
                            JsonType::Null => {
                                *cur_param = JsonType::Object(Default::default());
                                if let JsonType::Object(temp) = cur_param {
                                    temp.insert(attr.clone(), Json::new(JsonType::Null));
                                    cur_param = temp.get_mut(attr.as_str()).unwrap().get_mut();
                                }
                            },
                            JsonType::Object(ref mut temp) => { 
                                temp.insert(attr.clone(), Json::new(JsonType::Null));
                                cur_param = temp.get_mut(attr.as_str()).unwrap().get_mut(); 
                            },
                            _ => {
                                return Err(std::io::Error::new(std::io::ErrorKind::Other, "must be object").into());
                            }
                        }
                        
                        //cur_param = temp.get_mut(&attr).unwrap().get_mut();
                    }
                    else {
                        match cur_param {
                            JsonType::Null => { 
                                *cur_param = JsonType::Vec(Default::default());
                                if let JsonType::Vec(temp) = cur_param {
                                    temp.push(Json::new(JsonType::Null));
                                    let len = temp.len() - 1;
                                    cur_param = temp[len].get_mut();
                                }
                            },
                            JsonType::Vec(ref mut temp) => {
                                temp.push(Json::new(JsonType::Null));
                                let len = temp.len() - 1;
                                cur_param = temp[len].get_mut();
                            },
                            _ => {
                                 return Err(std::io::Error::new(std::io::ErrorKind::Other, "must be array").into());
                            }
                        }
                    }

                    key_right_opt = key[key_left..].find("%5B");
                }

                //println!("cur_param:{cur_param}");

                left = left + right + 1;
                right_opt = form_data[left..].find("&");
                let value;
                if let Some(right) = right_opt {
                    value = &form_data[left..][..right];
                    left = left + right + 1;
                }
                else
                {
                    value = &form_data[left..];
                }

                *cur_param = match urldecode(value.to_string()) {
                    Ok(decode_str) => JsonType::String(decode_str),
                    Err(e) => return Err(e),
                };

                if right_opt == None { break;}

                right_opt = form_data[left..].find("=");
            }

            Ok(json_obj)
        }
    }

    #[derive(Debug, Default, Clone)]
    pub struct HttpRequest {
        method: String,
        uri: String,
        query_string: String,
        version: String,
        header: std::collections::HashMap<String, String>,
        body: std::vec::Vec<u8>,
    }

    impl HttpRequest {
        pub fn set_method<T: Into<String>>(&mut self, method: T) {
            self.method = method.into(); 
        }

        pub fn get_method(&self) -> &String {
            &self.method 
        }

        pub fn set_uri<T: Into<String>>(&mut self, uri: T) {
            let uri: String = uri.into();
            if let Some(pos) = uri.find("?") {
                let (uri, query_string) = uri.split_at(pos);
                self.uri = uri.to_string();
                self.query_string = query_string[1..].to_string();
            }
            else {
                self.uri = uri;
            }
        }

        pub fn get_uri(&self) -> &String {
            &self.uri
        }

        pub fn get_query_string(&self) -> &String {
            &self.query_string 
        }

        pub fn get_version(&self) -> &String {
            &self.version
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

        pub fn get_header_keys(&self) -> impl Iterator<Item = &String> {
            self.header.keys()
        }

        pub fn set_body(&mut self, buffer: std::vec::Vec<u8>) {
            self.body = buffer;

            self.insert_header("content-length", self.body.len().to_string());
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

    #[derive(Debug)]
    #[derive(PartialEq)]
    enum RequestReaderState {
        Method,
        URI,
        Version,
        Header,
        Body,
        End,
    }

    #[derive(Debug)]
    pub struct HttpRequestReader {
        //当前等待读取的内容
        cur_state: RequestReaderState, 
        http_request: HttpRequest,
        cache: Vec<u8>,
    }

    impl HttpRequestReader {
        pub fn new() -> Self {
            Self {
                cur_state: RequestReaderState::Method,
                http_request: Default::default(),
                cache: Default::default(),
            } 
        }

        pub fn read(&mut self, mut buf: Vec<u8>) -> Result<&mut Self, BacktraceError> {
            if self.cur_state == RequestReaderState::Body && buf.len() == 0 {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "read body but buf not change").into());
            }

            self.cache.append(&mut buf);
            drop(buf);

            //println!("cur_state:{:?}", self.cur_state);
            if self.cache.len() == 0 { return Ok(self); }

            match self.cur_state {
                RequestReaderState::Method => {
                    if let Some(pos) = self.cache.iter().position(|&item| item == b' ') {
                        self.http_request.set_method(std::str::from_utf8(&self.cache[..pos])?);
                        self.cur_state = RequestReaderState::URI;
                        self.cache = self.cache[pos + 1 ..].to_vec();
                
                        return self.read(vec![]);
                    }
                },
                RequestReaderState::URI => {
                    if let Some(pos) = self.cache.iter().position(|&item| item == b' ') {
                        self.http_request.set_uri(std::str::from_utf8(&self.cache[..pos])?);
                        self.cur_state = RequestReaderState::Version;
                        self.cache = self.cache[pos + 1 ..].to_vec();
                
                        return self.read(vec![]);
                    }
                },
                RequestReaderState::Version => {
                    if let Some(pos) = self.cache.iter().position(|&item| item == b'\n') {
                        self.http_request.set_version(std::str::from_utf8(&self.cache[..pos - 1])?);
                        self.cur_state = RequestReaderState::Header;
                        self.cache = self.cache[pos + 1 ..].to_vec();
                
                        return self.read(vec![]);
                    }
                },
                RequestReaderState::Header => {
                    //println!("*{:?}*", std::str::from_utf8(&self.cache)?);
                    if let Some(pos) = self.cache.iter().position(|&item| item == b'\n') {
                         //println!("pos:{pos}");
                        if pos == 1 {
                            self.cache = self.cache[pos + 1..].to_vec();
                            let body_len = self.http_request.get_body_len();
                            if body_len > 0 {
                                self.cur_state = RequestReaderState::Body; 
                                return self.read(vec![]);
                            }
                            else {
                                self.cur_state = RequestReaderState::End;
                                return Ok(self)
                            }
                
                        }
                        else if let Some(key_pos) = self.cache.iter().position(|&item| item == b':') {
                            let content = std::str::from_utf8(&self.cache[..pos])?;
                            self.http_request.insert_header(content[..key_pos].trim(), content[key_pos + 1..pos].trim());
                            self.cache = self.cache[pos + 1 ..].to_vec();
                
                            return self.read(vec![]);
                        }
                        else {
                            return Ok(self);
                            //return Err(std::io::Error::new(std::io::ErrorKind::Other, "header not correct").into());
                        }
                    } 
                    else {
                        return Ok(self); 
                            //return Err(std::io::Error::new(std::io::ErrorKind::Other, "header not correct").into());
                    }
                },
                RequestReaderState::Body => {
                    //println!("cache_len:{}",self.cache.len());
                    //println!("in body:{}", urldecode(std::str::from_utf8(self.cache.as_slice())?)?);
                    if self.cache.len() == self.http_request.get_header("content-length").unwrap().parse::<usize>()? {
                         self.http_request.set_body(std::mem::take(&mut self.cache));
                        self.cur_state = RequestReaderState::End;
                    }
                
                    return Ok(self);
                },
                RequestReaderState::End => {
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, "read finish").into());
                }
            } 

            return Err(std::io::Error::new(std::io::ErrorKind::Other, "should not in here").into());
        }

        pub fn is_finished(&self) -> bool {
            return self.cur_state == RequestReaderState::End;
        }

        pub fn get_request(self) -> Result<HttpRequest, BacktraceError> {
            if !self.is_finished() { return Err(std::io::Error::new(std::io::ErrorKind::Other, "read not finish").into());}

            return Ok(self.http_request);
        }
    }

    #[derive(Copy, Clone, Debug, TryFromPrimitive)]
    #[repr(u16)]
    pub enum HttpResponseStatusCode {
        OK = 200,
        NotFound = 404,
        BadRequest = 400,
        Unauthorized = 401,
        Found = 302,
        UnsupportedMediaType = 415,
        InternalServerError = 500,
    }

    #[derive(Debug)]
    pub struct HttpResponse {
        version: String, 
        status_code: HttpResponseStatusCode,
        status_desc: String,
        header: std::collections::HashMap<String, String>,  
        body: std::vec::Vec::<u8>,
    }

    impl HttpResponse {
        pub fn new(code: HttpResponseStatusCode) -> Self {
            Self { 
                version: String::from("HTTP/1.1"), 
                status_code: code,
                status_desc: format!("{:?}", code),
                header: Default::default(),
                body: Default::default(),
            }
        }

        fn get_file(path: &str) -> Result<HttpResponse, BacktraceError> {
            println!("path:{}", path);

            let file_res = std::fs::OpenOptions::new().read(true).open(path);
            match file_res {
                Ok(mut file) => {
                    let mut buffer = vec![0u8; file.metadata()?.len() as usize];

                    let len = file.read(buffer.as_mut_slice())?;
                    assert_eq!(len, buffer.len());

                    let mut response = HttpResponse::new(HttpResponseStatusCode::OK);
                    
                    let file_type = *path.split('.').collect::<Vec<&str>>().last().unwrap();
                    let content_type = match file_type {
                        "html" => "text/html; charset=utf-8",
                        "js" => "text/javascript",
                        "css" => "text/css",
                        "ico" => "image/x-icon",
                        _ => "charset=utf-8",
                    };
                    response.insert_header("content-type", content_type);

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

        pub fn view(path: &str) -> Result<HttpResponse, BacktraceError> {
            let mut view: String = String::from("views/");
            view += path;

            Self::get_file(view.as_str())
        }

        pub fn get_root_file(uri: &str) -> Result<HttpResponse, BacktraceError> {
            let mut root: String = String::from("wwwroot");
            root += uri;

            Self::get_file(root.as_str())
        }

        pub fn json(json_val: Json) -> Self { 
            let data = json_val.to_string().as_bytes().to_vec();

            let mut response = Self {
                version: String::from("HTTP/1.1"), 
                status_code: HttpResponseStatusCode::OK,
                status_desc: format!("{:?}", HttpResponseStatusCode::OK),
                header: Default::default(),
                body: Default::default(), 
            };

            response.set_body(data);
            response.insert_header("content-type", "application/json; charset=utf-8");

            return response;
        }

        pub fn set_status_code(&mut self, code: HttpResponseStatusCode) {
            self.status_code = code;
        }

        pub fn get_status_code(&self) -> HttpResponseStatusCode {
            self.status_code
        }

        pub fn set_status_desc<T: Into<String>>(&mut self, desc: T) {
            self.status_desc = desc.into();
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

        pub fn set_version<T: Into<String>>(&mut self, version: T) {
            self.version = version.into()
        }

        pub fn get_version(&self) -> &str {
            self.version.as_str()
        } 
    }

    #[derive(Debug, PartialEq)]
    enum ResponseReaderState {
        Version,
        StatusCode,
        StatusDesc,
        Header,
        Body,
        End,
    }

    #[derive(Debug)]
    pub struct HttpResponseReader {
        cur_state: ResponseReaderState,
        response: HttpResponse,
        cache: Vec<u8>,
    }

    impl Default for HttpResponseReader {
        fn default() -> Self {
            Self {
                cur_state: ResponseReaderState::Version,
                response: HttpResponse::new(HttpResponseStatusCode::OK), 
                cache: Vec::<u8>::new(),    
            }
        }
    }

    impl HttpResponseReader {
        pub fn read(&mut self, mut buf: Vec<u8>) -> Result<&mut Self, BacktraceError> {
            self.cache.append(&mut buf);

            match self.cur_state {
                ResponseReaderState::Version => {
                    if let Some(pos) = self.cache.iter().position(|&item| item == b' ') {
                        self.response.set_version(std::str::from_utf8(&self.cache[..pos])?);
                        self.cache = self.cache[pos + 1..].to_vec();
                        self.cur_state = ResponseReaderState::StatusCode;
                        
                        return self.read(vec![]);
                    }
                },
                ResponseReaderState::StatusCode => {
                    if let Some(pos) = self.cache.iter().position(|&item| item == b' ') {
                        let code_str = std::str::from_utf8(&self.cache[..pos])?;
                        let code_u16 = code_str.parse::<u16>()?;
                        let enum_code = HttpResponseStatusCode::try_from(code_u16)?;
                        self.response.set_status_code(enum_code);
                        self.cache = self.cache[pos + 1..].to_vec();
                        self.cur_state = ResponseReaderState::StatusDesc;
                        
                        return self.read(vec![]);
                    }
                },
                ResponseReaderState::StatusDesc => {
                    if let Some(pos) = self.cache.iter().position(|&item| item == b'\n') {
                        self.response.set_status_desc(std::str::from_utf8(&self.cache[..pos - 1])?);
                        self.cache = self.cache[pos + 1..].to_vec();
                        self.cur_state = ResponseReaderState::Header;
                        
                        return self.read(vec![]);
                    }
                },
                ResponseReaderState::Header => {
                    if let Some(return_pos) = self.cache.iter().position(|&item| item == b'\n') {
                        let attr = &self.cache[..return_pos];

                        if attr[0] == b'\r' { 
                            self.cur_state = ResponseReaderState::Body;
                            self.cache = self.cache[return_pos + 1..].to_vec();
                        }
                        else if let Some(split_pos) = attr.iter().position(|&item| item == b':') {
                            let key = &attr[..split_pos];
                            let value = &attr[split_pos + 1..return_pos - 1];

                            self.response.insert_header(std::str::from_utf8(&key)?.trim().to_lowercase(), std::str::from_utf8(&value)?.trim());
                            self.cache = self.cache[return_pos + 1..].to_vec();
                        }

                        return self.read(vec![]);
                    } 
                },
                ResponseReaderState::Body => {
                    let (_key, val) = self.response.get_header().find(|(key, _val)| key.as_str() == "content-length").ok_or(std::io::Error::new(std::io::ErrorKind::Other, "not have content-length"))?;
                    if self.cache.len() == val.parse::<usize>()? {
                        self.response.set_body(std::mem::take(&mut self.cache));
                        self.cur_state = ResponseReaderState::End;
                    }
                },
                ResponseReaderState::End => {
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, "is finish").into());
                },
            }

            Ok(self)
        }

        pub fn is_finished(&self) -> bool {
            return self.cur_state == ResponseReaderState::End;
        }

        pub fn get_response(self) -> Result<HttpResponse, BacktraceError> {
            if !self.is_finished() { return Err(std::io::Error::new(std::io::ErrorKind::Other, "read not finish").into());}

            return Ok(self.response);
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

        pub fn call(&self, method: &str, url: &str, request: &HttpRequest) -> Result<HttpResponse, BacktraceError> {
            let link = self.routes.get(method).ok_or(std::io::Error::new(std::io::ErrorKind::Other, "route have not register method"))?;
            let func = link.get(url).ok_or(std::io::Error::new(std::io::ErrorKind::Other, "route have not register url"))?;

            let def = String::new();
            let content_type = request.get_header("content-type").unwrap_or(&def);

            let json = if content_type.find("application/json").is_some() { 
                Json::parse(std::str::from_utf8(request.get_body())?)?
            }
            else if content_type.find("application/x-www-form-urlencoded").is_some() {
                Json::parse_form_data(std::str::from_utf8(request.get_body())?)?
            }
            else {
                Json::parse_form_data(request.get_query_string())?
            };

            Ok(func(json))
        }
    }
    
    pub struct HttpServer {
        socket: std::net::TcpListener,
        router: std::sync::Arc<Router>,
        use_ssl: bool,
    }

    impl HttpServer {
        async fn send_response(stream: &mut impl Write, response: &HttpResponse) -> Result<(), BacktraceError> {
            let head_content = format!
                                (
                                    "{version} {status_code} {status_desc}\r\n{header}\r\n", 
                                    version = response.get_version(), 
                                    status_code = response.get_status_code() as u16,
                                    status_desc = format!("{:?}", response.get_status_code()),
                                    header = (|| {
                                        let mut result = String::new();
                                        for (key, val) in response.get_header() {
                                            result += format!("{}: {}\r\n", key, val).as_str();
                                        }

                                        return result;
                                    })()
                                );

            let wait_write = vec![head_content.as_bytes(), response.get_body().as_slice()];
            for cur_buf in wait_write {
                let mut had_write = 0usize;
                while had_write < cur_buf.len() {
                    had_write += stream.write(&cur_buf[had_write..cur_buf.len()])?;
                }
            }
            //stream.flush().await.unwrap();

            Ok(())
        }

        fn handle_request(router: &Router, request: &HttpRequest) -> Result<HttpResponse, BacktraceError> {
            let method = request.get_method();
            let uri = request.get_uri();

            println!("handle_request:{}, {}", method, uri);
            if router.contains_url(method, uri) {
                router.call(method, uri, request)
            }
            else {
                let response = HttpResponse::get_root_file(uri)?;

                Ok(response)
            }
        }

        async fn handle_accept(stream: &mut impl Read) -> Result<HttpRequest, BacktraceError> {
            let mut reader = HttpRequestReader::new();

            while !reader.is_finished() {
                let mut buf = [0u8; 1024];
                let buf_size = stream.read(&mut buf)?;
                //println!("{:?}", std::str::from_utf8(&buf[0..buf_size])?);

                if buf_size == 0 { return Err(std::io::Error::new(std::io::ErrorKind::Other, "socket close").into()); }

                reader.read(buf[..buf_size].to_vec())?;
            }

            //println!("{:#?}", http_request);

            //println!("finish!!!!!!!!!!!!");

            return Ok(reader.get_request()?)
        }

        async fn accept_process(use_ssl: bool, router: std::sync::Arc::<Router>, stream: std::net::TcpStream) -> Result<(), BacktraceError> {
            enum HttpStream<'a> {
                TCP(&'a std::net::TcpStream),
                SSL(openssl::ssl::SslStream::<&'a std::net::TcpStream>),
            }

            impl<'a> Read for HttpStream<'a> {
                fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
                    match self {
                        HttpStream::TCP(stream) => stream.read(buf),
                        HttpStream::SSL(stream) => stream.read(buf),
                    }
                }
            }

            impl<'a> Write for HttpStream<'a> {
                fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
                    match self {
                        HttpStream::TCP(stream) => stream.write(buf),
                        HttpStream::SSL(stream) => stream.write(buf),
                    }
                }

                fn flush(&mut self) -> Result<(), std::io::Error> {
                    match self {
                        HttpStream::TCP(stream) => stream.flush(),
                        HttpStream::SSL(stream) => stream.flush(),
                    }
                }
            }

            println!("accept_process...");
            
            let mut wrap_stream: HttpStream = if use_ssl { 
                let mut acceptor = openssl::ssl::SslAcceptor::mozilla_intermediate(openssl::ssl::SslMethod::tls())?;
                acceptor.set_private_key_file("key.pem", openssl::ssl::SslFiletype::PEM)?;
                acceptor.set_certificate_chain_file("cert.pem")?;
                acceptor.check_private_key()?; 
                let acceptor = acceptor.build();
                    HttpStream::SSL(acceptor.accept(&stream)?)
                }
                else {
                    HttpStream::TCP(&stream)
                };

            loop {
                match wrap_stream {
                    HttpStream::TCP(ref stream) => println!("ip:{} handle_accept...", stream.peer_addr()?),
                    HttpStream::SSL(ref stream) => println!("ip:{} handle_accept...", stream.get_ref().peer_addr()?),
                }
                let request_res = Self::handle_accept(&mut wrap_stream).await;

                match request_res {
                    Err(e) => {
                        if e.err_desc == "socket close" {
                            return Ok(());
                        }
                        else {
                            return Err(e);
                        }
                    },
                    Ok(request) => {
                        let mut response = Self::handle_request(&router, &request)?;

                        //println!("{:#?}", request);

                        if request.get_header("accept-encoding").unwrap_or(&String::new()).split(",").find(|&item| item == "gzip").is_some() {
                            //println!("boy use gzip");
                            let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
                            encoder.write_all(response.get_body())?;
                            response.set_body(encoder.finish()?);
                            response.insert_header("content-encoding", "gzip");
                        }

                        response.insert_header("connection", "close");
                        //response.insert_header("connection", "keep-alive");
                        //response.insert_header("keep-alive", "timeout=5");

                        Self::send_response(&mut wrap_stream, &response).await?;
                    }
                }
                //暂时不支持长链接
                break;
            }

            Ok(())
        }

        pub async fn new(ip_addr: &str) -> Result<Self, BacktraceError> {
            let bind_res = std::net::TcpListener::bind(ip_addr); 
            match bind_res {
                Ok(val) => { 
                    Ok(
                        Self {
                            socket: val,
                            router: std::sync::Arc::new(Router::new()),
                            use_ssl: false,
                        }
                    )
                },
                Err(e) => Err(e.into()),
            }
        }

        pub fn get_router(&mut self) -> &mut std::sync::Arc<Router> {
            return &mut self.router;
        }

        pub fn use_ssl(&mut self, is_use: bool) -> &mut Self{
            self.use_ssl = is_use;
            return self;
        }

        pub async fn listen(&self) -> Result<(), BacktraceError> { 
            println!("incoming...");
            println!("{:?}", self.router.routes.get("POST").unwrap().keys());
            for stream_res in self.socket.incoming() {			
                let router_copy = std::sync::Arc::clone(&self.router);
                let use_ssl = self.use_ssl;
                match stream_res {
                   Ok(stream) => {
                        stream.set_read_timeout(Some(std::time::Duration::from_millis(5000)))?;
                        stream.set_write_timeout(Some(std::time::Duration::from_millis(5000)))?;

						let _handle = async_std::task::spawn(async move {
                        	if let Err(e) = Self::accept_process(use_ssl, router_copy, stream).await {
                                if e.err_desc != "future timed out" {
								    println!("{}", e);
                                }
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

    #[derive(Default)]
    pub struct HttpClient {
    }

    impl HttpClient {
        fn get_request_header_string(request: &HttpRequest) -> String {
            format!("{method} {uri}{query_string} {version}\r\n\
                    {headers}\
                    \r\n",
                    method = request.get_method(),
                    uri = request.get_uri(), 
                    query_string = request.get_query_string(),
                    version = request.get_version(),
                    headers = request.get_header_keys().map(|key| {
                        let val = request.get_header(key).unwrap();
                        //let mut new_key = key.clone();
                        //new_key.replace_range(0..1, &key.as_str()[0..1].to_uppercase());
                        format!("{key}: {val}\r\n", )
                    }).collect::<String>()
            )
        }

        pub fn send<T: Into<String>>(&self, address: T, mut request: HttpRequest) -> Result<(), BacktraceError> {
            let mut address: String = address.into();
            let host: String;
            let port: String;
            let uri: String;
            let mut protocol = String::default();

            for item in ["http://", "https://"] {
                if let Some((left, right)) = address.split_once(item) {
                    if !left.is_empty() {panic!("")}
                    protocol = item.to_string();
                    address = right.to_string();
                    break;
                }
            }

            if protocol.is_empty() { return Err(std::io::Error::new(std::io::ErrorKind::Other, "not match protocol!").into());}

            let host_port: Vec<&str>;
            if let Some((left, right)) = address.split_once('/') {
                host_port = left.split(':').collect::<Vec<&str>>();
                uri = format!("/{right}");
            }
            else {
                host_port = address.split(':').collect::<Vec<&str>>();
                uri = String::from("/");
            }

            if host_port.len() == 1 {
                host = host_port[0].to_string();
                port = match protocol.as_str() {
                    "http://" => String::from("80"),
                    "https://" => String::from("443"),
                    _ => return Err(std::io::Error::new(std::io::ErrorKind::Other, "not match protocol!").into()),
                };
            }
            else {
                host = host_port[0].to_string();
                port = host_port[1].to_string();
            }


            //println!("protocol:{protocol},host:{host},port:{port},uri:{uri}");

            request.insert_header("host", host.as_str());
            request.set_uri(uri);


            let request_str = Self::get_request_header_string(&request);

            //println!("{request_str}");

            let mut stream = std::net::TcpStream::connect(format!("{host}:{port}"))?;
            let mut ssl_stream = None;
            let write_obj: Option<&mut (dyn Write)> = match protocol.as_ref() {
                "https://" => {
                    let connector = openssl::ssl::SslConnector::builder(openssl::ssl::SslMethod::tls())?.build();
                    ssl_stream = Some(connector.connect(host.as_str(), &stream)?);
                    Some(ssl_stream.as_mut().unwrap())
                },
                _ => {
                    Some(&mut stream)
                }
            };

            let write_obj = write_obj.unwrap();

            write_obj.write_all(request_str.as_bytes())?;
            write_obj.write_all(request.get_body())?;

            let read_obj: &mut (dyn Read) = match protocol.as_ref() {
                "https://" => ssl_stream.as_mut().unwrap(),
                _ => { 
                    drop(ssl_stream);
                    &mut stream
                }
            };

            let mut buffer = [0u8; 1024];
            let mut response_reader = HttpResponseReader::default();

            while !response_reader.is_finished() {
                let buffer_size = read_obj.read(&mut buffer)?;
                response_reader.read(buffer[0..buffer_size].to_vec())?;
            }

            //println!("{}", std::str::from_utf8(&response_reader.get_response()?.get_body())?);

            Ok(())
        }
    }
}

#[cfg(test)]
mod json_tests {
    use super::*;

    #[test]
    fn set_and_get_val() {
        let mut json = web::Json::new(crate::web::JsonType::Object(Default::default()));

        json.set_val("asd", web::Json::new(crate::web::JsonType::i64(123)));

        assert_eq!(i64::from(json.get_val("asd").unwrap()), 123);
    }

    #[test]
    fn get_null_val() {
        let json = web::Json::new(crate::web::JsonType::Null);

        assert!(json.get_val("asd").is_none()); 
    }

    #[test]
    fn parse_json() {
        let json_str = " { \"a\": 123, \"fgfgfg\": 444.2, \"complex\": { \"son\": 123}, \"c\": \"aaad\" } ";
        let json = web::Json::parse(json_str).unwrap();

        println!("test_display:\n{}", json);

        assert_eq!(i64::from(json.get_val("a").unwrap()), 123);
    }

    #[test]
    fn form_data() {
        let temp = "w=%E4%B8%AD%E6%96%87test%F0%9F%92%96&foo=bar&lalala=123&ids%5B%5D=1&ids%5B%5D=2&ids%5B%5D=3";
        //let temp = "ids%5B%5D=1&ids%5B%5D=2";
        println!("string:\n{}", temp);

        let json = web::Json::parse_form_data(temp).unwrap();

        //println!("decode:{}", web::urldecode(json.to_string().as_str()).unwrap()); 

        assert_eq!("中文test💖", String::from(json.get_val("w").unwrap()));
        assert_eq!("bar", String::from(json.get_val("foo").unwrap()));
        assert_eq!("123", String::from(json.get_val("lalala").unwrap()));

        match json.get_val("ids").unwrap() {
            web::JsonType::Vec(arr) => {
                assert_eq!("1", String::from(arr[0].get()));
                assert_eq!("2", String::from(arr[1].get()));
                assert_eq!("3", String::from(arr[2].get()));
            },
            _ => { assert!(false, "wrong type")} 
        }


        println!("form_data:\n{}", json);
    }

    #[test]
    fn null_json() {
        let json_str = "null";

        let json = web::Json::parse(json_str).unwrap();
        
        println!("test_display:\n{}", json);
        assert_eq!("null", String::from(&*json.get()));
    }

	#[test]
	fn array_json_number() {
		let json_str = "[  703,26,322]";

		let json = web::Json::parse(json_str).unwrap();

		assert_eq!(web::JsonType::i64(703), *json.index(0).unwrap().get());
		assert_eq!(web::JsonType::i64(26), *json.index(1).unwrap().get());
		assert_eq!(web::JsonType::i64(322), *json.index(2).unwrap().get());
	}

	#[test]
	fn array_json_decimal() {
		let json_str = "[7.4,20.3,3.6]";

		let json = web::Json::parse(json_str).unwrap();

		assert_eq!(web::JsonType::f64(7.4), *json.index(0).unwrap().get());
		assert_eq!(web::JsonType::f64(20.3), *json.index(1).unwrap().get());
		assert_eq!(web::JsonType::f64(3.6), *json.index(2).unwrap().get());
	}

    #[test]
    fn array_json_string() {
		let json_str = "[\"first\", \"second\", \"第三个\"]";

        let json = web::Json::parse(json_str).unwrap();

        println!("{}", json);
		assert_eq!(web::JsonType::String("first".into()), *json.index(0).unwrap().get());
		assert_eq!(web::JsonType::String("second".into()), *json.index(1).unwrap().get());
		assert_eq!(web::JsonType::String("第三个".into()), *json.index(2).unwrap().get());
    }

    #[test]
    fn parse_complex_json_1() {
        let json_str = " { \"a\": 123, \"c\": \"a\\\"ha\\\"aad\", \"fgfgfg\": 444.2, \"complex\": { \"son\": 123}, \"zzz\": null} ";
        let json = web::Json::parse(json_str).unwrap();

        println!("test_display:\n{}", json);

        assert_eq!(String::from(&*json.get_val("c").unwrap()), "a\"ha\"aad");
    }

    #[test]
    fn parse_complex_json_2() {
		let json_str = "[{\"a\": 123, \"bb\": \"abc\"}, {\"a\": 456}, {\"a\": 789}]";

        let json = web::Json::parse(json_str).unwrap();

        assert_eq!(web::JsonType::i64(123), *json.index(0).unwrap().get_val("a").unwrap());
        assert_eq!(web::JsonType::String("abc".into()), *json.index(0).unwrap().get_val("bb").unwrap());

        assert_eq!(web::JsonType::i64(456), *json.index(1).unwrap().get_val("a").unwrap());

        assert_eq!(web::JsonType::i64(789), *json.index(2).unwrap().get_val("a").unwrap());
    }

    #[test]
    fn parse_complex_json_3() {
        let json_str = "{\"www\": \"中文♥\", \"lalala\": [1, 2, 3]}";

        let json = web::Json::parse(json_str).unwrap();

        assert_eq!(web::JsonType::String("中文♥".into()), *json.get_val("www").unwrap());
    }
}

#[cfg(test)]
mod urldecode {
    use super::*;

	#[test]
	fn test_decode() {
		let code = "name=123&aaa=444&www=%E4%B8%AD%E6%96%87test%F0%9F%92%96";
		
		let result = web::urldecode(code);
		assert_eq!("name=123&aaa=444&www=中文test💖", result.unwrap());
	}
}

#[cfg(test)]
mod router_tests {
    use super::*;

    fn test_response(param: web::Json) -> web::HttpResponse {
        println!("test_response!!!param:{}", param);

        return web::HttpResponse::new(web::HttpResponseStatusCode::OK);
    }

    #[test]
    fn call_back_register() {
        let mut router = web::Router::new(); 

        println!("add:{:p}", test_response as fn(_) -> _);
        router.register_url("GET".to_string(), "asd".to_string(), &test_response);

        let mut request = web::HttpRequest::default();
        request.set_body("{\"a\": 123123}".as_bytes().to_vec());
        request.insert_header("content-length", request.get_body().len().to_string());

        router.call("GET", "asd", &request).unwrap();
    }

    #[test]
    fn contains_uri() {
        let mut router = web::Router::new();

        router.register_url("POST", "/asd", &test_response);
        assert_eq!(true, router.contains_url("POST", "/asd"));
        assert_eq!(false, router.contains_url("GET", "/asd"));
        assert_eq!(false, router.contains_url("POST", "asd"));
    }
}

#[cfg(test)]
mod request_tests {
    use super::*;

    #[test]
    fn http_request() {
        let mut reader = web::HttpRequestReader::new();

        let content = "GET / HTTP/1.1\r\n\
        Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7\r\n\
        Accept-Encoding: gzip, deflate\r\n\
        Accept-Language: zh-CN,zh;q=0.9\r\n\
        Cache-Control: max-age=0\r\n\
        Connection: keep-alive\r\n\
        Host: 43.139.67.232:8888\r\n\
        Upgrade-Insecure-Requests: 1\r\n\
        User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36\r\n\r\n";

        println!("{:?}", content);

        reader.read(content.as_bytes().to_vec()).unwrap();

        assert_eq!(true, reader.is_finished());

    }
}

#[cfg(test)]
mod http_client_tests {
    use super::*;

    #[test]
    fn get() {
        let client = web::HttpClient::default();

        let mut request = web::HttpRequest::default();
        request.set_method("POST");
        request.set_version("HTTP/1.1");
        request.insert_header("accept", "application/json");
        request.insert_header("user-agent", "none");
        //request.insert_header("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7");
        request.insert_header("cache-control", "no-cache");
        request.insert_header("pragma", "no-cache");
        request.insert_header("content-type", "application/json");
        request.insert_header("authorization", "WECHATPAY2-SHA256-RSA2048 mchid=\"1900000001\",...");

        let json = web::Json::new([
            ("appid".into(), "wxd678efh567hg6787".into()),
            ("mchid".into(), "1230000109".into()),
            ("description".into(), "Image形象店-深圳腾大-QQ公仔".into()),
            ("out_trade_no".into(), "1217752501201407033233368018".into()),
            ("notify_url".into(), "https://www.weixin.qq.com/wxpay/pay.php".into()),
            ("amount".into(), [
                ("total".into(), 100.into()), 
                ("currency".into(), "CNY".into())
            ].into()),
            ("payer".into(), [
                ("openid".into(), "oUpF8uMuAJO_M2pxb1Q9zNjWeS6o\t".into())].into()),
            ("scene_info".into(), [
                ("payer_client_ip".into(), "14.23.150.211".into()), 
                ("h5_info".into(), [("type".into(), "iOS".into())].into())
            ].into()),
        ].into());

        request.set_body(json.to_string().into());
        client.send("https://api.mch.weixin.qq.com/v3/pay/transactions/h5", request.clone()).unwrap();
        client.send("https://www.baidu.com:443", request.clone()).unwrap();
        client.send("http://www.baidu.com:80/", request).unwrap();
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
