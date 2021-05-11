use alloc::string::String;
use alloc::vec::Vec;

/// This represents a Path in the kernel.
/// It is a simple wrapper around a `String`
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Path(String);

impl Path {
    /// Returns a new empty `Path`
    pub fn new() -> Self {
        Self(String::new())
    }
    /// Builds owned `Path` from `String` reference
    pub fn from(s: &str) -> Self {
        Self(s.into())
    }
    /// returns new owned `String`
    pub fn to(&self) -> String {
        self.0.clone()
    }
    /// returns owned `String`
    pub fn owned_to(self) -> String {
        self.0
    }
    /// Slices the `Path` and returns a `Vec<String>`
    pub fn slice(&self) -> Vec<String> {
        let sliced = self
            .to()
            .split('/')
            .map(String::from)
            .collect::<Vec<String>>();
        sliced
    }
    /// Appends a segment to the `Path`
    pub fn push_str(&mut self, s: &str) {
        self.0.push('/');
        self.0.push_str(&(*s))
    }
    /// Get the parent `Path`
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
    /// Returns the most upper segment.
    pub fn get_name(&self) -> String {
        self.slice().get(0).unwrap_or(&String::from("")).clone()
    }
    /// Builds a `Path` from a slice of `String`
    pub fn from_sliced(sliced: &[String]) -> Self {
        let mut path = Self::new();
        for e in sliced {
            path.push_str(e);
        }
        path
    }
}
impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}
impl Clone for Path {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
