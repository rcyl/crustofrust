pub mod cell;
pub mod refcell;
pub mod rc;

use std::borrow::Cow;

/*
Let's say this function is looking to escape special charecters,
it is very wasteful to return String for a string like "foo" where
no special characters are present
fn escape<'a>(s: &str) -> String {
    ' => \'
    " => \"
    foo => foo
} 

fn escape<'a>(s: &'a str) -> Cow<'a, str> {
    if already_escaped(s) {
        Cow::Borrowed(s)
    } else {
        let string = s.to_string();
        // do something to string(add \)
        Cow::Owned(string)
    }
}

In the std library, from_utf8_lossy returns Cow
impl String
    fn from_utf8_lossy(bytes: &[u8]) -> Cow<'_, str> {
        //if all the bytes is valid utf8 can just cast to str and return
        if valid_utf8(bytes) {
            Cow::Borrowed(bytes as &str)
        } else {
            let mut bts = Vec::from(bytes);
            for bts {
                // replace with INVALID CHAT utf-8 symbol if not valid utf-8
            }
            Cow::Owned(bts as String)
        }
    }
*/
