# Design Idea: Register Slice

At the moment, tock-registers does not have a way to retrieve a slice of a
RegisterArray. I spent some time thinking about the implementation of a
register slice feature, and realized that it is complex enough to implement that
it might not be worth building. However, in case we decide to build it down the
road, I want to write down some of the conclusions that I have reached so we
don't have to repeat this work in the future.

## Basic idea

Currently, if a driver wants to pass around a subslice of a register array, it
has to pass around two things:

1. The RegisterArray instance pointing to the entire array.
1. The range of indices that this slice represents (start and end, or start and
   length).

In most cases, this will take three words: one for the RegisterArray's address
and two for the index range. However, if we add a slice API into
`RegisterArray`, we can reduce that by one for RealRegisterArray to:

1. An Address pointing to the first element of the subslice.
1. The length of the subslice.

This design implements that, falling back to the three-word design for the unit
test environment (where presumably the overhead is either required or at least
perfectly fine).

## RegisterSlice trait

We introduce a new public trait `RegisterSlice` with a couple of
implementations:

```rust
pub trait RegisterSlice {
    type Element: Copy;

    // Methods omitted because I did not make it as far as designing them.
}

// This doesn't actually need to be public, as its only used within the provided
// implementation of `RegisterArray::slice`.
struct DefaultSlice<L: Len, A: RegisterArray<L>> {
    array: A,
    start: usize,
    len: usize,
    _phantom: PhantomData<L>,
}

impl<L: Len, A: RegisterArray<L>> DefaultSlice<L, A> {
    pub(crate) fn new<R: RangeBounds<usize>>(array: A, range: R) -> Option<Self> {
        let end = match range.end_bound() {
            Bound::Included(&end) if end < L::LEN => end + 1,
            Bound::Included(_) => return None,
            Bound::Excluded(&end) if end <= L::LEN => end,
            Bound::Excluded(_) => return None,
            Bound::Unbounded => L::LEN,
        };
        let start = match range.start_bound() {
            Bound::Included(&start) if start <= end => start,
            Bound::Included(_) => return None,
            Bound::Excluded(&start) if start < end => start + 1,
            Bound::Excluded(_) => return None,
            Bound::Unbounded => 0,
        };
        Some(DefaultSlice {
            array,
            start,
            len: end - start,
            _phantom: PhantomData,
        })
    }
}

impl<L: Len, A: RegisterArray<L>> RegisterSlice for DefaultSlice<L, A> {
    type Element = A::Element;
}

pub(crate) struct RealSlice<Element: Span> {
    address: Element::Address,
    _phantom: RealPhantom,
}

impl<Element: Span> RealSlice<Element> {
    pub(crate) unsafe fn new<R: RangeBounds<usize>>(address: Element::Address, range: R) -> Option<Self> {
        // Implementation omitted. Would use address.byte_add() to compute the
        // new address.
    }
}
```

## Generic associated slice type

The slice type varies between RealRegisterArray and other RegisterArray
instances, so we want to add it as a generic associated type. To prevent every
fake RegisterArray from having to specify the slice type, we need to specify a
default:

```rust
use core::ops::RangeBounds;

pub trait RegisterArray {
    // Rest of RegisterArray trait omitted.

    type Slice: RegisterSlice<Element = Self::Element> = DefaultSlice<Self::Element>;
    fn slice<R: RangeBounds<usize>>(self, range: R) -> Option<Self::Slice> {
        DefaultSlice::new(self, range)
    }
}
```

However, default values for associated types in traits are not stable yet.
Therefore, sadly, we have to use `impl Trait` syntax instead:

```rust
use core::ops::RangeBounds;

pub trait RegisterArray {
    // Rest of RegisterArray trait omitted.

    fn slice<R: RangeBounds<usize>>(self, range: R) -> Option<impl RegisterSlice<Element = Self::Element>> {
        DefaultSlice::new(self, range)
    }
}
```

The impl for RealRegisterArray will look like:

```rust
impl<Element: Span, L: Len> RegisterArray<L> for RealRegisterArray<Element, L> {
    // Other items omitted

    fn slice<R: RangeBounds<usize>>(self, range: R) -> Option<impl RegisterSlice<Element = Self::Element>> {
        unsafe {
            RealSlice::new(self.address, range)
        }
    }
}
```

## Interaction with array iterators

Assuming we implement array iterators first, we might consider using a single
iterator type for RegisterArray and RegisterSlice. Maybe.
