pub trait VecAsserts<T> {
    fn assert_forall(&self, f: impl Fn(&&T) -> bool) 
    where T: std::fmt::Debug;
    
    fn assert_exists(&self, f: impl Fn(&&T) -> bool) 
    where T: std::fmt::Debug;
    
    fn assert_eq(&self, other: &Vec<T>) 
    where T: std::fmt::Debug + PartialEq;
    
    fn assert_len(&self, len: usize) 
    where T: std::fmt::Debug;

    fn assert_empty(&self) 
    where T: std::fmt::Debug;

    fn assert_not_empty(&self) 
    where T: std::fmt::Debug;    
}

#[cfg(not(tarpaulin_include))]
impl<T> VecAsserts<T> for Vec<T> {
    fn assert_forall(&self, f: impl Fn(&&T) -> bool) where T: std::fmt::Debug {
        for (i, x) in self.iter().enumerate() {
            assert!(f(&x), "failed for element {:?} (index {})", x, i);
        }
    }
    fn assert_exists(&self, f: impl Fn(&&T) -> bool) where T: std::fmt::Debug {
        for x in self {
            if f(&x) {
                return;
            }
        }
        panic!("no element satisfied the predicate");
    }
    fn assert_eq(&self, other: &Vec<T>) where T: std::fmt::Debug + PartialEq {
        let other = other.iter();
        for (i, (x, y)) in self.iter().zip(other).enumerate() {
            assert_eq!(x, y, "failed for index {} ({:?} != {:?})", i, x, y);
        }
    }
    fn assert_len(&self, len: usize) where T: std::fmt::Debug {
        let actual = self.len();
        assert_eq!(len, actual, "expected {} elements, was {}", len, actual);
    }
    fn assert_empty(&self) where T: std::fmt::Debug {
        assert!(self.is_empty(), "expected empty vector, was {:?}", self);
    }
    fn assert_not_empty(&self) where T: std::fmt::Debug {
        assert!(!self.is_empty(), "expected non-empty vector");
    }
}