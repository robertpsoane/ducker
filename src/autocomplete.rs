#[derive(Debug)]
pub struct Autocomplete<'a> {
    possibles: Vec<&'a str>,
}

impl<'a> Autocomplete<'a> {
    pub fn from(mut possibles: Vec<&'a str>) -> Self {
        possibles.sort();
        Self { possibles }
    }

    pub fn get_completion(&self, current: &str) -> Option<String> {
        // Only want to attempt completion for commands with at least 2 letters
        // as 1 letter commands are possible and it could create confusion
        if current.len() < 2 {
            return None;
        }

        for possible in &self.possibles {
            if possible.starts_with(current) {
                let candidate = String::from(*possible);
                return Some(candidate);
            }
        }

        None
    }
}
