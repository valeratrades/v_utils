use serde_json::Value;

pub fn strip_nulls(value: &mut Value) {
	match value {
		Value::Object(map) => {
			map.retain(|_, v| !v.is_null());
			for v in map.values_mut() {
				strip_nulls(v);
			}
		}
		Value::Array(arr) =>
			for v in arr.iter_mut() {
				strip_nulls(v);
			},
		_ => {}
	}
}
pub fn filter_nulls(mut value: Value) -> Value {
	strip_nulls(&mut value);
	value
}
