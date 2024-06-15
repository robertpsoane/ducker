#[derive(Debug, Clone)]
pub struct PageHelp {
    name: String,
    inputs: Vec<(String, String)>,
}

impl PageHelp {
    pub fn new(name: String) -> Self {
        Self {
            name: name,
            inputs: vec![],
        }
    }

    pub fn add_input(mut self, trigger: String, description: String) -> Self {
        self.inputs.push((trigger, description));
        self
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_inputs(&self) -> Vec<(String, String)> {
        self.inputs.clone()
    }
}
