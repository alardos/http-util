pub struct HttpParam {
    name: String,
    value: HttpParamValue,
}

#[derive(PartialEq, Eq, Debug)]
pub enum HttpParamValue {
    Singular(String),
    List(Vec<String>),
}

