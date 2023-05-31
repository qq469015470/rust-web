#[cfg(test)]
mod tests {
    #[test]
    fn set_and_get_val() {
        let mut json = crate::web::Json::new();

        json.set_val("asd", crate::web::JsonType::i64(123));

        assert_eq!(i64::from(json.get_val("asd").unwrap()), 123);
    }

    #[test]
    fn get_null_val() {
        let mut json = crate::web::Json::new();

        assert_eq!(json.get_val("asd"), None);
    }

    #[test]
    fn parse_json() {
        let json_str = " { \"a\": 123, \"c\": \"aaad\", \"fgfgfg\": 444.2, \"complex\": { \"son\": 123} } ";
        let mut json = crate::web::Json::parse2(json_str).unwrap();

        println!("test_display:\n{}", json);

        assert_eq!(i64::from(json.get_val("a").unwrap()), 123);
    }
}

mod web {
    #[allow(dead_code)]
    #[derive(PartialEq)]
    #[derive(Debug)]
    #[allow(non_camel_case_types)]
    pub enum JsonType {
        i64(i64),
        f64(f64),
        String(String),
        Vec(Vec<JsonType>),
        Json(Json),
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

    impl From<&mut JsonType> for String {
        fn from(item: &mut JsonType) -> Self {
            match item {
                JsonType::String(val) => val.to_string(),
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
                JsonType::Json(val) => write!(f, "{}", val),
            }
        }
    }

    #[allow(dead_code)]
    #[derive(PartialEq)]
    #[derive(Debug)]
    #[derive(Default)]
    pub struct Json {
        attr: std::collections::HashMap<String, JsonType>
        //val: Option<JsonType>,
    }

    impl Json {
        pub fn new() -> Self {
            return Json { ..Default::default()};
        }

        pub fn set_val<T: Into<String>>(&mut self, key: T, val: JsonType) {
            self.attr.insert(key.into(), val);
        }

        pub fn get_val(&mut self, key: &str) -> Option<&mut JsonType> {
            self.attr.get_mut(key)
        }

        pub fn parse2(mut json_str: &str) -> Result<Json, &'static str> {
            println!("parse2 function");
            
            let mut result = Json::new();
            let mut cur_json: &mut Json = &mut result;

            #[derive(PartialEq)]
            enum ReadState {
                KeyNameSignal,
                KeyName,
                Val,
                ValNumber,
                ValNumberDecial,
                ValString,
                WaitSignal(char), 
                EndSignal,
            }

            let mut cur_state = ReadState::Val;
            let mut cache = String::new();
            let mut key_name = String::new();
            let mut val_str: String;

            for c in json_str.chars() {
                if c == ' ' { continue; }

                println!("loop [{}]", c);

                match cur_state {
                    ReadState::KeyNameSignal => {
                        if c != '"' { return Err("not key name signal") };

                        cur_state = ReadState::KeyName;
                    },
                    ReadState::KeyName => {
                        match c {
                            '"' => {
                                cur_state = ReadState::WaitSignal(':');
                                key_name = cache;
                                println!("key:[{}]", key_name);
                                cache = String::new();
                            },
                            _ => {
                                cache.push(c); 
                            }
                        }
                    },
                    ReadState::Val => {
                        match c {
                            '{' => {
                                cur_state = ReadState::KeyNameSignal;

                                let mut temp = Json::new(); 
                                cur_json.set_val(key_name.to_string(), JsonType::Json(temp));

                                match cur_json.get_val(&key_name).unwrap() {
                                    JsonType::Json(val) => {
                                        cur_json = val;
                                    },
                                    _ => {
                                        return Err("not json obj");
                                    }
                                }

                                key_name = String::new();
                            },
                            '"' => {
                                cur_state = ReadState::ValString; 
                            }, 
                            _ => {
                                if c.is_numeric() { 
                                    cur_state = ReadState::ValNumber; 
                                    cache.push(c);
                                }
                                else { return Err("??? wtf"); }
                            },
                        }
                    },
                    ReadState::ValNumber => {
                        match c {
                            ',' | '}' => {
                                if c == ',' { cur_state = ReadState::KeyNameSignal; }
                                else { cur_state = ReadState::EndSignal; }

                                val_str = cache;
                                cache = String::new();

                                cur_json.set_val(key_name, JsonType::i64(val_str.parse::<i64>().unwrap()));
                                key_name = String::new();
                            },
                            '.' => {
                                cur_state = ReadState::ValNumberDecial;
                            },
                            _ => {
                                if !c.is_numeric() { return Err("not number");}

                                cache.push(c); 
                            },
                        }
                    },
                    ReadState::ValNumberDecial => {
                        match c {
                            ',' | '}' => {
                                if c == ',' { cur_state = ReadState::KeyNameSignal; }
                                else { cur_state = ReadState::EndSignal; }

                                val_str = cache;
                                cache = String::new();

                                cur_json.set_val(key_name, JsonType::f64(val_str.parse::<f64>().unwrap()));
                                key_name = String::new();
                            },
                            _ => {
                                if !c.is_numeric() && c != '.' { return Err("not number");}

                                cache.push(c); 
                            },
                        }
                    },
                    ReadState::ValString => {
                        match c {
                            '"' => { 
                                val_str = cache;
                                cur_json.set_val(key_name, JsonType::String(val_str));

                                cache = String::new();
                                key_name = String::new();

                                cur_state = ReadState::EndSignal;
                            },
                            _ => { cache.push(c) },
                        }
                    },
                    ReadState::WaitSignal(val) => {
                        if c != val { return Err("wait signal not correct"); }

                        match c {
                            ':' => {
                                cur_state = ReadState::Val;
                            },
                            _ => {
                            }
                        }
                    },
                    ReadState::EndSignal => {
                        match c {
                            '}' => {},
                            ',' => cur_state = ReadState::KeyNameSignal,
                            _ => {
                                return Err("not end signal"); 
                            },
                        }
                    }
                }
            }

            return Ok(result);
        }

        pub fn parse(mut json_str: &str) -> Result<Json, &'static str>{ 
            let mut result = Json::new();

            enum ValType {
                Number
            }

            //let val_type;

            for c in json_str.chars() {
                match c {
                    ' ' => continue,
                    '{' => {},
                    '"' => {}, 
                    val => {
                        if val.is_digit(10) {
                            //val_type = ValType::Number;
                        }
                    },
                }
            }


            return Ok(result);
        }
    }

    impl std::fmt::Display for Json {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{{")?;
            for (count, (key, val)) in self.attr.iter().enumerate() {
                if count != 0 { write!(f, ", ")?; }
                write!(f, "{:?}: {}", key, val)?;
            }
            write!(f, "}}")
        }
    }
}
