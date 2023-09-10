

/*******************************************************************************
// Vec

Vec is composed of RawVec, the implementation fo allocation for Vec::new is 
at rust/library/alloc/src/raw_vec.rs
if mem::size_of::<T> is 1, it s 8, for size less that 1024, it is 4
and anything bigger is 1, which implies you should try to use Vec::with_capacity
vec![] macro does Vec::with_capacity

capacity is the max amount of elements you can have without relloacating

Vec is supposed to be contiguous, so removing an element would require a shift
of the other elements
swap_remove take the last item in the vector and replaces it with the element
to be removed, so it is a lot more efficient if you don't care about the ordering

unsafe Vec::set_len can be used to extend the length of the vector into memory that you
have initialized yourself.
use case: image you are doing ffi with c and you are passing the pointer to C (with as_mut_ptr)
the C writes into the element, let's say 500 elements. with set_len you can tell the compiler
you know that the first 500 elements vector has been written to. 

Vec::retain is a smarter way if removing items from the vector and it not that expensive

Vec::leak will return a &'a mut [T] (a 'static mutable reference to a slice/memory)
The lifetime is whatever you want assign to be, for example
let static_ref: &'static mut [usize] = x.leak();
use case: let's say you have vector of configuration data to be shared around,
can just leak it and pass the reference(which is static) around. This is ok since
configuration data lives for the entire duration of the program

small_vec crate can be used if your collection isnt that big, because for small collections
it uses an array internally (stack allocated), 
and when the collection is bigger it uses the vanilla Vec (heap allocated)
there is a cost of using small_vec, because there is a branch to check whether it is 
stack or heap allocated, and if they entire small_vec is passed into a function, the 
whole thing is copied versus Vec which just passes the heap allocated memory

Vec only implements IntoInterator trait and not Interator trait because Vec is not an iterator.
Tterators needs to keep track of items that was just yielded and Vec does not have those 
fields to do so. 

******************************************************************************
// VecDeque

VecDeque is implenmented with a ring buffer.
Ring buffers can act as both a stack (LIFO) and a queue (FIFO).

VecDeques are less efficient because it wraps to the front. 
For every read, the processor fetches more memory that is has to for caching reasons
However, if we are close to the end, the extra memory fetched by the processor 
extends beyond the bound of the ring buffer, whereas we are wrapping to the front,
and therefore the caching is less efficient.

Indexing of the items is also more complicated which leads to the inefficiency

We also cannot deref a VecDeque into a slice, because a slice is a contiguous area of memory.
As a result, it has it's own implementation of binary search (binary search is a method from 
Deref<Target=[T]>, implemented on slices)

VecDeque::make_contiguous, moves the start position to 0 and the end position to
however many elements are in the collection and doesn't wrap around. Therefore,
make_contigous can return a slice [T]

Can also use as_slices to get slice (it returns 2 slices)
1. The first slice is from the start or end or if it wraps around, it is from
start to end of collection
2. The second slice is empty if it doesn't wrap around and if it does, it is from
end to start. Check DOCS!

******************************************************************************
// LinkedList

From the docs: It is almost always better to use Vec or VecDeque because array-based containers 
are generally faster, more memory efficient, and make better use of CPU cache.

Main benefit from linked list is if you need to unlink them in the middle
and link them back later on


******************************************************************************
// Map and Sets (Both HashMap/BTreeMap and HashSet/BtreeSet)
In the following paragraph Set is short for either HashSet or BTreeSet and
Map is short for either HashMap or BTreeMap 

Set is composed of Map internally. The key to the map is the set type,
and the value doesn't matter. If you have a map, the keys are unique which means
the keys are a set. To check if a key is in a set by using a map, check that 
the key is in the map. 

Jon uses BTreeSet because he could show the source code easier without multiple indirections

Sets can use bit wise operations, for example and, or and xor

let a = BTreeSet::from([1, 2, 3]);
let b = BTreeSet::from([2, 3, 4]);

let result = &a & &b;
assert_eq!(result, BTreeSet::from([2, 3]));

******************************************************************************
// HashMap

When a map is generated, a random number will be used to seed the hashes
for access to the map, so it will be difficult for an attacker to figure out
the keys to generate a hash collision

Std library's implementation tries to be secure as default but also less performant,
therefore there is an option to use your own hash function with BuildHasher trait

shrink_to_fit can be used to measure how many items are currently in the data structure,
and reduce the capacity to only fit those items. For hashmaps, the hashes fromy they key
after this operation might also change

remove_entry<Q>(&mut self, k: &Q) -> Option<(K, V)>where
    K: Borrow<Q>,
    Q: Hash + Eq + ?Sized,
remove_entry returns both the key and the value, given the key
Let's say the key to the map is String, you can lookup the value via str.
You can do remove_entry with a str and it returns String as the key, which
is valuable if you want to reuse that allocation for something else

entry gives you a reference to the bucket inside the hashmap. Entry is an enum
which is either occupied or vacant. In a typical case, you might write,
if let Some(v) = map.get_mut(k) {
    // do something to the value
} else {
    map.insert(k, v)
}
Get_mut() and insert() involves two rounds of bucket lookups and hashing 
whereas entry only does it once and returns pointer to the bucket, which you can 
modify. You can chain it with or_insert() which returns a &'a mut V

Std library's hashmap is now implemented  sing the hashbrown crate, because
it is superior over the initial implementation. So the implementation of hashmap
no longer lives in the standard library but in the crate hashbrown.
You can choose to use hashbrown directly, but it's fairly similar to the 
standard library except for its hashing function which is slighty faster.

Or you can use hashbrown in environments without std such as embedded systems and kernel

******************************************************************************
// BTreeMap

BTreeMap does not require that the key is hashable, instead it requires that 
the key is orderable. 

In binary search trees, every node is an allocation and you have to store
a lot of things
Node {
    key: K
    value: V
    left_ptr: &'a mut Node
    right_ptr: &'a mut Node
}
By right, you only need store size of K + V, but for BST you needs to 
store the size of (K + V + 2 * ptr_size) and this overhead grows very fast

BTrees (see 1:48:54) for illustration  
Node in BTree store B number of items, overhead is less because you store less
pointers. BTree is also more cache friendly

B=6 in rust/library/alloc/src/collections/btree/node.rs

You use this when you want the keys to the sorted. BTreeMaps also have a more 
compact memory allocation generally. You can also extend a btree map
with another btree map, and no new allocations are necessary just need to shift them
around to balance the map 

******************************************************************************
// BinaryHeap
 
BinaryHeap (see 2:12:00) for illustration  
Max heap - Highest values T go to the top
Min heap - Lowest values T go to the top

The only operation in a heap, is get the max or min (besides insertion)
The std lib's binary heap is a max heap.
You can also get the same functionality with BTreeMap but a BinaryHeap
is more efficiently implemented for getting the max value

Binary heap allows duplicate values

To do min heap, you can either change your ordering algo 
or wrap your values into std::cmp::Reverse before pushing onto the heap

type () is called the unit type
from the docs,
use std::io::*;
let data = vec![1, 2, 3, 4, 5];
let res: Result<()> = data.iter()
    .map(|x| writeln!(stdout(), "{x}"))
    .collect();
assert!(res.is_ok());
In this example, we don't really care what value is returned writeln!, just whether
there were errors. The Ok type is unit

for concurrent hashmaps, can use dashmap, flurry or crossbeam crates

when iterating indexmap, you get the keys back in insertion order 

if you know all your keys at compile time, you can use phf_shared crate 
(phf stands for perfect hash map) to make sure that all your keys hash to exactly one
bucket, so you dont need linear probing or etc

*******************************************************************************/


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn it_works() {
        let x = vec![1, 2, 3];
        let static_ref: &'static mut [u32] = x.leak();
        static_ref[0] += 1;
        assert_eq!(static_ref, &[2, 2, 3]);


        let a = HashSet::from([1, 2, 3]);
        let b = HashSet::from([2, 3, 4]);
        let result = &a & &b;
        assert_eq!(result, HashSet::from([2, 3]));

    }
}
