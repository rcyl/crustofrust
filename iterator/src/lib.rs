mod one {
    trait Iterator {
        type Item; // this is an associated type
        fn next(&mut self) -> Option<Self::Item>;
    }

    // This style is known as generic type parameter to trait
    // trait Iterator<Item> {
    //     fn next(&mut self) -> Option<Item>;
    // }
    /* why is the former used rather than the latter?
    rule of thumb is that you use an associated type if you
    only expect there will only be one implementation of the
    trait for a given type

    Use the latter if multiple implementations might make sense for a given type
    */
    #[test]
    #[ignore]
    fn test_vec() {
        // let vs = vec![1, 2, 3];
        // for v in vs {
        //     // consumes vs, owned v
        // }
        // for v in vs.iter() {
        //     //borrows vs, & to v
        // }
        // for v in &vs {
        //     // equivalent to vs.iter()
        // }
    }
}

mod two {
    // flatten (in the std lib) only recurses only level down, NOT turtles all the way!
    pub struct Flatten<O> {
        outer: O,
    }

    pub fn flatten<I>(iter: I) -> Flatten<I> {
        Flatten::new(iter)
    }

    impl<O> Flatten<O> {
        fn new(iter: O) -> Self {
            Flatten { outer: iter }
        }
    }
    impl<O> Iterator for Flatten<O>
    where
        O: Iterator,
    {
        type Item = O::Item;
        // But this isnt quite right, we want O::Item::Item
        // item of the item
        fn next(&mut self) -> Option<Self::Item> {
            None
        }
    }
}

mod three {
    #[derive(Debug, PartialEq)]
    pub struct Flatten<O> {
        outer: O,
    }

    pub fn flatten<I>(iter: I) -> Flatten<I> {
        Flatten::new(iter)
    }

    impl<O> Flatten<O> {
        fn new(iter: O) -> Self {
            Flatten { outer: iter }
        }
    }
    impl<O> Iterator for Flatten<O>
    where
        O: Iterator,
        // means the item needs to implement iterator as well
        O::Item: IntoIterator,
    {
        type Item = <O::Item as IntoIterator>::Item;
        fn next(&mut self) -> Option<Self::Item> {
            // and_then can be used for both result and option
            self.outer.next().and_then(|inner| inner.into_iter().next())
        }
    }

    #[test]
    fn empty() {
        assert_eq!(flatten(std::iter::empty::<Vec<()>>()).count(), 0);
    }

    #[test]
    fn empty_wide() {
        assert_eq!(
            flatten(vec![Vec::<()>::new(), vec![], vec![]].into_iter()).count(),
            0
        );
    }

    #[test]
    fn one() {
        assert_eq!(flatten(std::iter::once(vec!["a"])).count(), 1);
    }

    #[test]
    #[ignore] //because it fails
    fn two() {
        assert_eq!(flatten(std::iter::once(vec!["a", "b"])).count(), 2);
    }

    #[test]
    fn two_wide() {
        assert_eq!(flatten(vec![vec!["a"], vec!["b"]].into_iter()).count(), 2);
    }
}

mod four {
    pub fn flatten<I>(iter: I) -> Flatten<I>
    where
        I: Iterator,
        I::Item: IntoIterator,
    {
        Flatten::new(iter)
    }

    pub struct Flatten<O>
    where
        O: Iterator,
        O::Item: IntoIterator,
    {
        outer: O,
        inner: Option<<O::Item as IntoIterator>::IntoIter>,
    }

    impl<O> Flatten<O>
    where
        O: Iterator,
        O::Item: IntoIterator,
    {
        fn new(iter: O) -> Self {
            Flatten {
                outer: iter,
                inner: None,
            }
        }
    }
    impl<O> Iterator for Flatten<O>
    where
        O: Iterator,
        O::Item: IntoIterator,
    {
        type Item = <O::Item as IntoIterator>::Item;
        fn next(&mut self) -> Option<Self::Item> {
            loop {
                if let Some(ref mut inner_iter) = self.inner {
                    if let Some(i) = inner_iter.next() {
                        return Some(i);
                    }
                    self.inner = None;
                }

                let next_inner_iter = self.outer.next()?.into_iter();
                self.inner = Some(next_inner_iter);
            }
        }
    }

    #[test]
    fn empty() {
        assert_eq!(flatten(std::iter::empty::<Vec<()>>()).count(), 0);
    }

    #[test]
    fn empty_wide() {
        assert_eq!(
            flatten(vec![Vec::<()>::new(), vec![], vec![]].into_iter()).count(),
            0
        );
    }

    #[test]
    fn one() {
        assert_eq!(flatten(std::iter::once(vec!["a"])).count(), 1);
    }

    #[test]
    fn two() {
        assert_eq!(flatten(std::iter::once(vec!["a", "b"])).count(), 2);
    }

    #[test]
    fn two_wide() {
        assert_eq!(flatten(vec![vec!["a"], vec!["b"]].into_iter()).count(), 2);
    }
}

/* Remove the need to call into_iter() from the caller */
mod five {
    pub fn flatten<I>(iter: I) -> Flatten<I::IntoIter>
    where
        I: IntoIterator,
        I::Item: IntoIterator,
    {
        Flatten::new(iter.into_iter())
    }

    pub struct Flatten<O>
    where
        O: Iterator,
        O::Item: IntoIterator,
    {
        outer: O,
        inner: Option<<O::Item as IntoIterator>::IntoIter>,
    }

    impl<O> Flatten<O>
    where
        O: Iterator,
        O::Item: IntoIterator,
    {
        fn new(iter: O) -> Self {
            Flatten {
                outer: iter,
                inner: None,
            }
        }
    }
    impl<O> Iterator for Flatten<O>
    where
        O: Iterator,
        O::Item: IntoIterator,
    {
        type Item = <O::Item as IntoIterator>::Item;
        fn next(&mut self) -> Option<Self::Item> {
            loop {
                if let Some(ref mut inner_iter) = self.inner {
                    if let Some(i) = inner_iter.next() {
                        return Some(i);
                    }
                    self.inner = None;
                }

                let next_inner_iter = self.outer.next()?.into_iter();
                self.inner = Some(next_inner_iter);
            }
        }
    }

    #[test]
    fn empty() {
        assert_eq!(flatten(std::iter::empty::<Vec<()>>()).count(), 0);
    }

    #[test]
    fn empty_wide() {
        assert_eq!(flatten(vec![Vec::<()>::new(), vec![], vec![]]).count(), 0);
    }

    #[test]
    fn one() {
        assert_eq!(flatten(std::iter::once(vec!["a"])).count(), 1);
    }

    #[test]
    fn two() {
        assert_eq!(flatten(std::iter::once(vec!["a", "b"])).count(), 2);
    }

    #[test]
    fn two_wide() {
        assert_eq!(flatten(vec![vec!["a"], vec!["b"]]).count(), 2);
    }
}

mod six {

    pub fn flatten<I>(iter: I) -> Flatten<I::IntoIter>
    where
        I: IntoIterator,
        I::Item: IntoIterator,
    {
        Flatten::new(iter.into_iter())
    }

    pub struct Flatten<O>
    where
        O: Iterator,
        O::Item: IntoIterator,
    {
        outer: O,
        inner: Option<<O::Item as IntoIterator>::IntoIter>,
    }

    impl<O> Flatten<O>
    where
        O: Iterator,
        O::Item: IntoIterator,
    {
        fn new(iter: O) -> Self {
            Flatten {
                outer: iter,
                inner: None,
            }
        }
    }

    impl<O> Iterator for Flatten<O>
    where
        O: Iterator,
        O::Item: IntoIterator,
    {
        type Item = <O::Item as IntoIterator>::Item;
        fn next(&mut self) -> Option<Self::Item> {
            loop {
                if let Some(ref mut inner_iter) = self.inner {
                    if let Some(i) = inner_iter.next() {
                        return Some(i);
                    }
                    self.inner = None;
                }

                let next_inner_iter = self.outer.next()?.into_iter();
                self.inner = Some(next_inner_iter);
            }
        }
    }

    impl<O> DoubleEndedIterator for Flatten<O>
    where
        // DoubleEndedIterator implements Iterator so dont have to declare that trait
        O: DoubleEndedIterator,
        O::Item: IntoIterator,
        // IntoIter is an associated type
        <O::Item as IntoIterator>::IntoIter: DoubleEndedIterator,
    {
        fn next_back(&mut self) -> Option<Self::Item> {
            loop {
                if let Some(ref mut inner_iter) = self.inner {
                    if let Some(i) = inner_iter.next_back() {
                        return Some(i);
                    }
                    self.inner = None;
                }

                let next_inner_iter = self.outer.next_back()?.into_iter();
                self.inner = Some(next_inner_iter);
            }
        }
    }

    #[test]
    fn reverse() {
        assert_eq!(
            flatten(std::iter::once(vec!["a", "b"])).rev().collect::<Vec<_>>(),
            vec!["b", "a"]
        );
    }

    #[test]
    fn reverse_wide() {
        assert_eq!(
            flatten(vec![vec!["a"], vec!["b"]]).rev().collect::<Vec<_>>(),
            vec!["b", "a"]
        );
    }
}

mod seven {
    pub fn flatten<I>(iter: I) -> Flatten<I::IntoIter>
    where
        I: IntoIterator,
        I::Item: IntoIterator,
    {
        Flatten::new(iter.into_iter())
    }

    // Can also do
    // pub trait IteratorExt: Iterator + Sized
    pub trait IteratorExt: Iterator {
        fn our_flatten(self) -> Flatten<Self>
        where 
            Self: Sized,
            Self::Item: IntoIterator;
    }
    
    // Blanket implentation for all T
    // This means all types that implement iterators
    // has IteratorExt trait implemented for it
    impl<T> IteratorExt for T
    where 
        T: Iterator 
    {
        fn our_flatten(self) -> Flatten<Self>
        where
            Self: Sized,
            Self::Item: IntoIterator,
        {
            flatten(self)
        }
    }

    pub struct Flatten<O>
    where
        O: Iterator,
        O::Item: IntoIterator,
    {
        // type O is stored in the struct, so the compiler needs to know
        // how big it is, there it has to implement Sized
        outer: O,
        front_iter: Option<<O::Item as IntoIterator>::IntoIter>,
        back_iter: Option<<O::Item as IntoIterator>::IntoIter>,
    }

    impl<O> Flatten<O>
    where
        O: Iterator,
        O::Item: IntoIterator,
    {
        fn new(iter: O) -> Self {
            Flatten {
                outer: iter,
                front_iter: None,
                back_iter: None,
            }
        }
    }

    impl<O> Iterator for Flatten<O>
    where
        O: Iterator,
        O::Item: IntoIterator,
    {
        type Item = <O::Item as IntoIterator>::Item;
        fn next(&mut self) -> Option<Self::Item> {
            loop {
                if let Some(ref mut front_iter) = self.front_iter {
                    if let Some(i) = front_iter.next() {
                        return Some(i);
                    }
                    self.front_iter = None;
                }

                if let Some(next_inner) = self.outer.next() {
                    self.front_iter = Some(next_inner.into_iter());
                }
                else {
                    // Abit blur here, refer to video at 1h mark
                    return self.back_iter.as_mut()?.next();
                }
            }
        }
    }

    impl<O> DoubleEndedIterator for Flatten<O>
    where
        // DoubleEndedIterator implements Iterator so dont have to declare that trait
        O: DoubleEndedIterator,
        O::Item: IntoIterator,
        // IntoIter is an associated type
        <O::Item as IntoIterator>::IntoIter: DoubleEndedIterator,
    {
        fn next_back(&mut self) -> Option<Self::Item> {
            loop {
                if let Some(ref mut back_iter) = self.back_iter {
                    if let Some(i) = back_iter.next_back() {
                        return Some(i);
                    }
                    self.back_iter = None;
                }
                
                if let Some(next_back_inner) = self.outer.next_back() {
                    self.back_iter = Some(next_back_inner.into_iter());
                }
                else {
                    return self.front_iter.as_mut()?.next_back();
                }
            }
        }
    }

    #[test]
    fn both_ends() {
        let mut iter = flatten(vec![vec!["a1", "a2", "a3"], vec!["b1", "b2", "b3"]]);
        assert_eq!(iter.next(), Some("a1"));
        assert_eq!(iter.next_back(), Some("b3"));
        assert_eq!(iter.next(), Some("a2"));
        assert_eq!(iter.next_back(), Some("b2"));
        assert_eq!(iter.next(), Some("a3"));
        assert_eq!(iter.next_back(), Some("b1"));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn inf() {
        // infinite, no allocation of memory required!
        let mut iter = flatten((0..).map(|i| 0..i));
        // 0 => 0..0 => empty, so iter moves ahead to the next
        // 1 => 0..1 => [0]
        // 2 => 0..2 => [0, 1]
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
    }

    #[test]
    fn deep() {
        //assert_eq!(flatten(vec![vec![vec![0, 1]]]).count(), 2);
        // need to flatten the flatten
        // first flatten yields [0, 1]
        // second flatten yield 0, 1
        assert_eq!(flatten(flatten(vec![vec![vec![0, 1]]])).count(), 2);
    }
    #[test]
    fn ext() {
                                    //iter() works too
        assert_eq!(vec![vec![0, 1]].into_iter().our_flatten().count(), 2);
    }

}

// The ?Sized means we can opt out from the implicit size requirement
// for example, when we store a Box
pub struct Flatten<O: ?Sized>
where 
    O: Iterator,
    O::Item: IntoIterator,
{
    outer: Box<O>
}