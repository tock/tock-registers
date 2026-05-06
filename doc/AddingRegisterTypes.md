# Adding Register Types

External crates can extend tock-registers to support register types that
tock-registers does not directly support. This is done through two mechanisms:
defining a new bus type, and defining new operations.

## Adding a new Bus

A Bus type must be `Copy` and should wrap a type that can store the register's
address (generally an integer or pointer type). For example, if you're creating
a new bus for registers that are accessed by a byte-sized address (this might be
seen on a SPI/I2C device that exposes a register-based interface), you would
add:

```rust
#[derive(Clone, Copy)]
pub struct MyBus {
    address: u8,
    // Used to make MyBus !Sync, which is not needed but can help prevent
    // misuse (for the same reason as why raw pointers are !Sync).
    _phantom: core::marker::PhantomData<*mut ()>,
}
```

You should then implement the following traits:

1. `Address`
1. `Bus<T>` for each value type `T` that this bus supports.
1. `BusRead<T>` and/or `BusWrite<T>` for each value type for which this bus
   should support the `Read`/`Write` operations.
1. `Send`, if you want to be able to use `RegisterSender` to move registers
   between threads. If the bus is thread-local, then do not implement `Send`.

Note that `Bus::PADDED_SIZE` should be the size of the value type `T` on the
*target* system, not on the system that we are currently compiling for. That is
so that offset tests pass when a `register_layouts!` invocation using the bus is
compiled for a host system (such as for unit tests). `Mmio32` is a good example
here: `Bus<usize>::PADDED_SIZE` is a constant 4, even when you compile for a
64-bit host system.

## Adding new operations

The `Read` and `Write` operations are not enough for every use case. When you
need another operation, you can add that operation to your bus.

When implementing a new operation, you can refer to the `read` and/or `write`
modules.

An operation requires three elements:

1. The operation trait itself. This is the trait that the user specifies when
   they invoke `register_layouts!`, and which they use when they are interacting
   with the generated registers.
1. The operation macro. This must have the same name and path as the operation
   trait (in practice, this means the operation trait must be exported in the
   crate root).
1. The bus-operation trait (generally named `Bus*` where `*` is the name of the
   operation). This trait is implemented on your bus, and is used by the macro
   to implement the operation trait.

The operation trait should be a subtrait of `Register`, and it should use
`<Self::DataType as DataType>::Value` as the value type for operations, and
should take `self` by value (`Register` is a subtrait of `Copy`). If the
underlying hardware operation is safe, then this trait's methods should be safe
as well.

The operation macro needs to have at least the following arms:

```rust
#[macro_export]
macro_rules! MyOperation {
    // Provides a real implementation of the trait.
    //
    // Arguments:
    //   name:     The name of the register struct that the trait should be
    //             implemented on. This is guaranteed to be a tuple struct that
    //             wraps a bus.
    //   datatype: The DataType for the register specified by the user.
    //   generics: Generic arguments attached to the operation trait. This is
    //             left blank if the trait does not have generics (resulting in
    //             two consecutive commas in the input).
    //   rest:     A catch-all so that register_layouts! can pass additional
    //             arguments in the future without breaking every existing
    //             operation macro.
    (real_impl, $name:ident, $datatype:ty, $(<$generic:path>)?, $($rest:tt)*) => {
        impl<B: Bus + /* Your Bus* trait */> $crate::MyOperation
            for $name<B>
        {
            // Your trait implementation goes here. Use self.address to retrieve
            // the register's address.
        }
    };
    // Catch-all case that emits nothing if register_layouts! invokes it with an
    // unknown first argument. This is so that we can add new functionality into
    // the operations macros without breaking backwards compatibility (though
    // register_layouts! would need to be compatible with this do-nothing
    // block).
    ($($unknown:tt)*) => {};
}
```

Your `Bus*` trait should be implemented on your bus type. You may need that
trait to be generic (as `BusRead` and `BusWrite` are) so that you can provide
separate implementations for each type that your bus supports. If your bus can
be sent between threads, then it should also be implemented for
`BorrowedBus<YourBusTrait>`.

## Using your new Bus and operation

Once you have implemented a Bus, you can use it as follows:

```rust
use my_bus_crate::{MyBus, MyOperation};

tock_registers::register_layouts! {
    #![buses(MyBus)]
    foo {
        0 => control: u8 { MyOperation },
    }
}
```
