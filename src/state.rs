use std::collections::HashMap;


#[derive(Debug)]
pub struct State {
    pub parent      : Option<String>,
    pub name        : Option<String>,
    pub entry       : Option<String>,
    pub exit        : Option<String>,
    pub transitions : HashMap<String, String>
}
impl State {
    pub fn new() -> Self {
        State {
            parent      : None,
            name        : None,
            entry       : None,
            exit        : None,
            transitions : HashMap::new()
        }
    }
}
