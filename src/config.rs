use std::{collections::HashMap, path::PathBuf};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Configuration {
    pub binds: HashMap<String, Bind>,
}

#[derive(Deserialize)]
pub struct Bind {
    pub description: Option<String>,
    pub suggest: Option<String>,
    pub on_down: Option<Action>,
    pub on_up: Option<Action>,
}

#[non_exhaustive]
#[derive(Deserialize, Clone)]
pub enum Action {
    Run(Run),
}

#[derive(Deserialize, Clone)]
pub struct Run {
    pub program: PathBuf,
    pub arguments: Option<Vec<String>>,
}
