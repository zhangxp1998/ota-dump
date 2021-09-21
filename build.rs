use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(default)]
pub struct BuildConfig {
    /// protobuf include dirs
    pub includes: Vec<String>,
    /// protobuf files
    pub files: Vec<String>,
    /// dir for generated code
    pub output: String,
    /// build options for serde support
    pub opts: Vec<BuildOption>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(default)]
pub struct BuildOption {
    /// scope of the attribute
    pub scope: String,
    /// description of the option
    pub description: String,
    /// extra attribute to put on generated data structure, for example: `#[derive(Serialize, Deserialize)]`
    pub attr: String,
    /// a list of paths you want to add the attribute
    pub paths: Vec<String>,
}

pub fn build_with_serde(json: &str) -> BuildConfig {
    let build_config: BuildConfig = serde_json::from_str(json).unwrap();

    let mut config = prost_build::Config::new();
    for opt in build_config.opts.iter() {
        match opt.scope.as_ref() {
            "bytes" => {
                config.bytes(&opt.paths);
                continue;
            }
            "btree_map" => {
                config.btree_map(&opt.paths);
                continue;
            }
            _ => (),
        };
        for path in opt.paths.iter() {
            match opt.scope.as_str() {
                "type" => config.type_attribute(path, opt.attr.as_str()),
                "field" => config.field_attribute(path, opt.attr.as_str()),
                v => panic!("Not supported type: {}", v),
            };
        }
    }

    if !build_config.output.is_empty() {
        fs::create_dir_all(&build_config.output).unwrap();
        config.out_dir(&build_config.output);
    }

    config
        .compile_protos(&build_config.files, &build_config.includes)
        .unwrap_or_else(|e| panic!("Failed to compile proto files. Error: {:?}", e));

    build_config
}

fn main() {
    let json = include_str!("./build_config.json");
    build_with_serde(json);
}
