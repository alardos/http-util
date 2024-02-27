use std::collections::HashMap;

use serde::Serialize;

pub trait ToJson {
    fn to_json(&self) -> Json;
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Json {
    Obj(HashMap<String, Json>),
    Arr(Vec<Json>),
    Str(String),
    Num(i64),
}

impl From<Json> for Vec<u8> {
    fn from(value: Json) -> Self {
        value.to_string().into_bytes()
    }
}

impl From<JsonObj> for Json {
    fn from(value: JsonObj) -> Self {
        Json::Obj(value.fields)
    }
}

impl From<Vec<Json>> for Json {
    fn from(value: Vec<Json>) -> Self {
        Json::Arr(value)
    }
}

impl FromIterator<Json> for Json {
    fn from_iter<T: IntoIterator<Item=Json>>(iter: T) -> Self {
        let mut wrapped = Json::Arr(vec![]);
        let Json::Arr(mut arr) = wrapped else { panic!() };
        iter.into_iter().for_each(|x| arr.push(x) );
        Json::Arr(arr)
    }
}

impl ToString for Json {
    fn to_string(&self) -> String {
        match self {
            Json::Obj(obj) => {
                let mut res = String::new();
                res.push('{');
                for (key, value) in obj.into_iter(){
                    res.push('"');
                    res.push_str(key);
                    res.push('"');
                    res.push(':');
                    res.push_str(&value.to_string());
                    res.push(',');
                };
                res.pop();
                res.push('}');

                res

            }
            Json::Arr(arr) => {
                let mut arr_content = arr.iter().map(|obj| obj.to_string()).collect::<Vec<String>>().join(",");
                arr_content.push(']');
                arr_content.insert(0,'[');
                arr_content
            }
            Json::Str(s) => format!("\"{s}\""),
            Json::Num(n) => n.to_string()
        }
    }
}



impl Clone for Json {
    fn clone(&self) -> Self {
        match self {
            Self::Obj(arg0) => Self::Obj(arg0.clone()),
            Self::Arr(arg0) => Self::Arr(arg0.clone()),
            Self::Str(arg0) => Self::Str(arg0.clone()),
            Self::Num(arg0) => Self::Num(arg0.clone()),
        }
    }
}


impl From<JsonArr> for Vec<u8> {
    fn from(val: JsonArr) -> Self {
        val.to_string().into_bytes()
    }
}

impl From<JsonObj> for Vec<u8> {
    fn from(val: JsonObj) -> Self {
        val.to_string().into_bytes()
    }
}

#[derive(Debug)]
pub struct JsonObj {
    fields: HashMap<String, Json>
}

impl JsonObj {
    pub fn new() -> JsonObj {
        JsonObj {
            fields: HashMap::new()
        }
    }

    pub fn push(&mut self, key: &str, value: impl Into<Json>) {
        self.fields.insert(key.to_string(), value.into());
    }
}

impl Clone for JsonObj {
    fn clone(&self) -> Self {
        Self { fields: self.fields.clone() }
    }
}

// impl From<String> for JsonObj {
//     fn from(value: String) -> Self {
//         json
//
//
//     }
// }

//    fn parse_obj(text: &str) {
//        let mut index = 0;
//        let mut key: String = String::new();
//        let mut on_key = false;
//        let mut expecting_key = true;
//        let mut on_value = false;
//        let mut expecting_value = false;
//
//        let mut iter = text.chars();
//
//        for c in iter {
//            if expecting_key {
//                iter.skip_while(|c| c == '"');
//                iter.next();
//                expecting_key = false;
//                on_key = true;
//            }
//
//            if on_key {
//                match c {
//                    '"' => {
//                        on_key = false;
//                        expecting_value = true;
//                    }
//                    _ => key.push(c),
//                }
//            }
//
//            if expecting_value {
//                match c {
//                    '"' => {
//                        expecting_value = false;
//                        on_value = true;
//                    }
//                    '{' => {
//                    
//                }
//        }
//        
//
//
//            }
//        }
//    }
//
//    fn parse_arr(text: &str) {
//
//    } 
//}

impl ToString for JsonObj {
    fn to_string(&self) -> String {
        let mut res = String::new();
        res.push('{');
        for field in &self.fields {
            res.push('"');
            res.push_str(field.0);
            res.push('"');
            res.push(':');
            if let Json::Str(s) = field.1 {
                res.push('"'); res.push_str(s); res.push('"');
            } else {
                res.push_str(&field.1.to_string());
            }
            res.push(',');
        };
        res.pop();
        res.push('}');

        res
    }
}

#[derive(Debug)]
pub struct JsonArr {
    items: Vec<JsonObj>
}
impl JsonArr {
    pub fn new() -> Self {
        JsonArr {
            items: vec![],
        }
    }
}

impl From<Vec<JsonObj>> for JsonArr {
    fn from(value: Vec<JsonObj>) -> Self {
        JsonArr { items: value }
    }
}

impl From<&Vec<JsonObj>> for JsonArr {
    fn from(value: &Vec<JsonObj>) -> Self {
        JsonArr { items: (*value).clone() }
    }
}

impl Clone for JsonArr {
    fn clone(&self) -> Self {
        Self { items: self.items.clone() }
    }
}

impl ToString for JsonArr {
    fn to_string(&self) -> String {
        let mut arr_content = self.items.iter().map(|obj| obj.to_string()).collect::<Vec<String>>().join(",");
        arr_content.push(']');
        arr_content.insert(0,'[');
        arr_content
    }
}

impl FromIterator<JsonObj> for JsonArr {
    fn from_iter<T: IntoIterator<Item=JsonObj>>(iter: T) -> Self {
        let mut arr = JsonArr::new();
        iter.into_iter().for_each(|x| arr.items.push(x));
        arr
    }
}

