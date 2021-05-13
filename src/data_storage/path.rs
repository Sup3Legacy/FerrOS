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

    /// Create an identical path
    pub fn duplicate(&self) -> Self {
        Path::from(&self.0.clone())
    }

    pub fn len(&self) -> usize {
        self.slice().len()
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
            let mut first = true;
            let mut res = String::new();
            for elt in sliced.iter() {
                if first {
                    first = false
                } else {
                    res.push('/')
                }
                res.push_str(elt);
            }

            Self::from(&res)
        }
    }
    /// Returns the most upper segment.
    pub fn get_name(&self) -> String {
        let sliced = self.slice();
        sliced.last()
            .unwrap_or(&String::from(""))
            .clone()
    }
    /// Builds a `Path` from a slice of `String`
    pub fn from_sliced(sliced: &[String]) -> Self {
        let mut path = String::new();
        let mut first = true;
        for e in sliced {
            if first {
                first = false;
            } else {
                path.push('/');
            }
            path.push_str(e);
        }
        Self::from(&path)
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
