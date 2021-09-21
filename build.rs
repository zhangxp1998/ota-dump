use prost_serde::build_with_serde;

fn main() {
    let json = include_str!("./build_config.json");
    build_with_serde(json);
}
