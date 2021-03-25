use alloc::string::String;
use alloc::vec::Vec;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Path(String);

impl Path {
    pub fn new() -> Self {
        Self(String::new())
    }
    pub fn from(s: &str) -> Self {
        Self(s.into())
    }
    // We might wanna to avoid cloning string everywhere...
    pub fn to(&self) -> String {
        self.0.clone()
    }
    pub fn owned_to(self) -> String {
        self.0
    }
    pub fn slice(&self) -> Vec<String> {
        let sliced = self
            .to()
            .split('/')
            .map(String::from)
            .collect::<Vec<String>>();
        sliced
    }
    pub fn push_str(&mut self, s: &str) {
        self.0.push('/');
        self.0.push_str(&(*s))
    }
    pub fn get_parent(&self) -> Self {
        let mut sliced = self.slice();
        if sliced.is_empty() {
            Self::from(&self.to())
        } else {
            sliced.pop();
            let mut res = String::new();
            for elt in sliced.iter() {
                res.push_str(elt);
            }
            Self::from(&res)
        }
    }
    pub fn get_name(&self) -> String {
        self.slice().get(0).unwrap_or(&String::from("")).clone()
    }
}
impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}
