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
                let len1 = inputs.len();
                //log::info!("KEY {key:?}");
                //log::info!("LEN1 {len1}");
                let mut def_vec = Vec::<String>::new();
                let val = map.get(key.as_str()).unwrap_or(&mut def_vec);
                let init_len = val.len();
                //log::info!("INIT MAP's VEC length {init_len}");
                inputs.extend_from_slice(val);
                let len2 = inputs.len();
                //log::info!("LEN2 {len2}");
                map.insert(key, inputs);
                return (len1 as u64, len2 as u64);
            }
            Err(_err) => return (0u64, 0u64),
        }
    }
    return (0u64, 0u64);
}
