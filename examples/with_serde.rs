use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ModelSpec {
    obj_file: PathBuf,
    normal_map_file: Option<PathBuf>,
    diff_texture_file: Option<PathBuf>,
    spec_texture_file: Option<PathBuf>,
}

#[derive(Serialize, Deserialize)]
pub struct SceneConfig {
    model: Vec<ModelSpec>,
}

fn main(){
    let x: SceneConfig = SceneConfig {
        model: vec![
            ModelSpec {
                obj_file: "whatever.rs".into(),
                normal_map_file: None,
                diff_texture_file: Some("diff_text.rs".into()),
                spec_texture_file: None,
            },
            ModelSpec {
                obj_file: "something.obj".into(),
                normal_map_file: None,
                diff_texture_file: None,
                spec_texture_file: Some("whatever.tga".into())
            }
    ]};

    let output = toml::to_string(&x).unwrap();
    std::fs::write("whatever.toml", output).unwrap();
}