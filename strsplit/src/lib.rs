// str -> [char], collection,sequences of chars
// &str -> &[char]
// [char] can be on heap, stack, static memory
// String -> Vec<char>, vec is heap allocated
// String -> &str (cheap -- AsRef)
// &str -> String (expensive -- Clone)

/* Single lifetime
#[derive(Debug, PartialEq)]
pub struct StrSplit<'a> {
    remainder: Option<&'a str>,
    delimeter: &'a str,
}
*/

// According to Jon, multiple lifetimes are usually not needed and are quite rare
// It will come out when there is a need to store multiple references and it is
// important that they are not the same, because you want to return one
// without tying it to the other
#[derive(Debug, PartialEq)]
pub struct StrSplit<'haystack, 'delimeter> {
    remainder: Option<&'haystack str>,
    delimeter: &'delimeter str,
}

// can use anonymous lifetimes <'_> to let compiler guess the lifetime
impl<'haystack, 'delimeter> StrSplit<'haystack, 'delimeter> {
    pub fn new(haystack: &'haystack str, delimeter: &'delimeter str) -> Self {
        Self {
            remainder: Some(haystack),
            delimeter,
        }
    }
}

// Can also do below to tell the compilter we dont care about the delimeter lifetime
// impl<'haystack> Iterator for StrSplit<'haystack, '_>
impl<'haystack, 'delimeter> Iterator for StrSplit<'haystack, 'delimeter> {
    type Item = &'haystack str;

    fn next(&mut self) -> Option<Self::Item> {
        /*
        let remainder = self.remainder?;
        for this code above self.remainder returns an option to a reference (&'a str)
        which is copied, which remains the remainder is not the same as
        the one inside the struct, which means we are modifiying some other reference 
        which is why the test did not halt
        */
        let remainder = self.remainder.as_mut()?;
        if let Some(next_delim) = remainder.find(self.delimeter) {
            let until_delimeter = &remainder[..next_delim];
            *remainder = &remainder[(next_delim + self.delimeter.len())..];
            Some(until_delimeter)
        } else {
            self.remainder.take()
        }
    }

    /* Second implementation
    fn next(&mut self) -> Option<Self::Item> {
        // if ref mut keyword is not used, remainder is moved out of self.remainder
        // ref mut remainder is &mut &'a str, and self.remainder is Option<&'a str>
        // Consider let Some(&mut remainder) = self.remainder
        // This would only match Option<&mut T>
        if let Some(ref mut remainder) = self.remainder {
            if let Some(next_delim) = remainder.find(self.delimeter) {
                let until_delimeter = &remainder[..next_delim];
                *remainder = &remainder[(next_delim + self.delimeter.len())..];
                Some(until_delimeter)
            } else {
                self.remainder.take()
                // take sets the option to none and returns the item wrapped by option
            }
        } else {
            None
        }
    }
    */

    /* First implementation
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next_delim) = self.remainder.find(self.delimeter) {
            let until_delimeter = &self.remainder[..next_delim];
            self.remainder = &self.remainder[(next_delim + self.delimeter.len())..];
            Some(until_delimeter)
        } else if self.remainder.is_empty()  {
            None
        } else {
            let rest = self.remainder;
            self.remainder = "";
            // &'a str      // 'static str
            // Why is ok to to assign the str with 'static lifetime to the
            // one with 'a liftime?
            // Because static lifetime extends to the end of the program
            // Can always assign a type that has a longer lifetime
            Some(rest)
        }
    }
    */
}

// Old implementation, explicit lifetimes
//fn until_char<'s>(s: &'s str, c: char) -> &'s str 

// Don't need to put any lifetimes because compiler knows that only
// one lifetime returned 
fn until_char(s: &str, c: char) -> &str {
    let delim = format!("{}", c);
    // When compiler is told that s and delim has the same lifetime 
    // it is going to use the shorter one
    StrSplit::new(s, &delim)
        .next()
        .expect("StrSplit always give at least one result")
}

fn until_char2(s: &str, c: char) -> &str {
    StrSplit2::new(s, c)
        .next()
        .expect("StrSplit always give at least one result")
}

/*****************/
pub trait Delimeter {
    fn find_next(&self, s: &str) -> Option<(usize, usize)>;
}

#[derive(Debug, PartialEq)]
pub struct StrSplit2<'haystack, D> {
    remainder: Option<&'haystack str>,
    delimeter: D,
}

impl<'haystack, D> StrSplit2<'haystack, D> 
where 
    D: Delimeter 
{
    pub fn new(haystack: &'haystack str, delimeter: D) -> Self {
        Self {
            remainder: Some(haystack),
            delimeter,
        }
    }
}

impl<'haystack, D> Iterator for StrSplit2<'haystack, D> 
where 
    D: Delimeter 
{
    type Item = &'haystack str;
    fn next(&mut self) -> Option<Self::Item> {
        let remainder = self.remainder.as_mut()?;
        if let Some((delim_start, delim_end)) = self.delimeter.find_next(remainder) {
            let until_delimeter = &remainder[..delim_start];
            *remainder = &remainder[delim_end..];
            Some(until_delimeter)
        } else {
            self.remainder.take()
        }
    }
}

impl Delimeter for &str {
    fn find_next(&self, s: &str) -> Option<(usize, usize)> {
        s.find(self).map(|start| (start, start + self.len()))
    }
}

impl Delimeter for char {
    fn find_next(&self, s: &str) -> Option<(usize, usize)> {
        s.char_indices().find(|(_, c)| c == self).map(|(start, _)| 
            // need len_utf8 on char instead of just adding 1
            (start, start + self.len_utf8())
        )
    }
}

/******************/
#[test]
fn until_char_test() {
    assert_eq!(until_char("hello world", 'o'), "hell");
    assert_eq!(until_char2("hello world", 'o'), "hell");
}

#[test]
fn it_works() {
    let haystack = "a b c d e";
    let letters: Vec<_> = StrSplit::new(haystack, " ").collect();
    // assert_eq! generates nicerror errors
    assert_eq!(letters, (vec!["a", "b", "c", "d", "e"]));
}

#[test]
fn tail() {
    let haystack = "a b c d ";
    let letters: Vec<_> = StrSplit::new(haystack, " ").collect();
    // assert_eq! generates nicerror errors
    assert_eq!(letters, (vec!["a", "b", "c", "d", ""]));
}
