use serde_json::Value;

pub(super) fn join_query(query: Vec<String>) -> String {
    query.join(" ")
}

pub(super) fn print_json(value: &Value) {
    match serde_json::to_string_pretty(value) {
        Ok(output) => println!("{output}"),
        Err(_) => println!("{value}"),
    }
}
