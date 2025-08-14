use crate::REQUEST_MAP;

pub fn get_inputs<'a>(k: &'a str) -> Vec<String> {
    if let Some(v) = REQUEST_MAP.get() {
        match v.write() {
            Ok(mut map) => return map.remove(k).unwrap_or_default(),
            Err(_err) => return vec![],
        }
    };
    return vec![];
}

pub fn add_inputs(key: String, mut inputs: Vec<String>) -> (u64, u64) {
    if let Some(v) = REQUEST_MAP.get() {
        match v.write() {
            Ok(mut map) => {
                //let key = String::from(k);
                let def_vec = Vec::<String>::new();
                let val = map.get(key.as_str()).unwrap_or(&def_vec);
                let init_len = inputs.len();
                inputs.extend_from_slice(val);
                map.insert(key, inputs.to_vec());
                return (init_len as u64, map.len() as u64);
            }
            Err(_err) => return (0u64, 0u64),
        }
    }
    return (0u64, 0u64);
}
