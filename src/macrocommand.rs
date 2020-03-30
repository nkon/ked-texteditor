use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct MacroCommand {
    pub name: String,
    pub arg: usize,
    pub argstr: String,
}

