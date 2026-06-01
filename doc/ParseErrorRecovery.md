# Design Idea: Error recovery during parsing

Currently, whenever the parsing logic encounters an error it immediately stops
parsing and returns that error. This has a couple drawbacks:

1. It results in generating only the compile errors, so any code that depends on
   the register module that was supposed to be generated will throw errors. If
   you make an incorrect tweak to registers in an existing driver, this buries
   the real error behind a mountain of "unknown module x" style errors.
1. It means you can only fix one error at a time. For projects where `cargo
   check` (or their build systems' equivalent) takes a while, this adds a
   significant delay.
1. It ran be bad for IDE integration, because any minor typo/error results in
   type checking failing throughout the file.

I (jrvanwhy) started reworking the parsing logic to allow for recovery from some
errors, but ultimately shelved it because it was too much work. I think this is
a desirable change that we want eventually, so I'm recording some of the
structure in this document so we don't have to repeat work.

## Design

We introduce a new enum:

```rust
/// `Result<T, Error>` only allows us to express two outcomes: perfect success, or immediate error.
/// However, an immediate error is a pretty harsh outcome: it stops parsing, which prevents the
/// macro from outputting more than one error at a time, and it prevents code generation, which
/// will result in many "unknown module" errors from the code that depends on the generated module.
/// Therefore, for any AST node with non-immediate errors, we parse into `Result<Outcome<T>,
/// Error>` instead. Note that because `syn::parse::Parse` always returns `Result<Self>`, we still
/// use `Result::Err` to communicate errors that should immediately stop parsing.
#[cfg_attr(test, derive(Debug))]
pub enum Outcome<T> {
    /// Full success (no errors)
    Ok(T),
    /// An error that does not stop parsing or code generation.
    #[allow(dead_code)]
    Continue(T, Error),
    /// An error that stops code generation but not parsing.
    #[allow(dead_code)]
    NoGenerate(Error),
}

/// API used to populate an Outcome. Generally, [`Parse`](syn::Parse) impls will use an
/// `Outcome<T>` to track their errors and to return early if an error prevents them from
/// generating a T (either an unrecoverable error or a NoGenerate error). On success, the Parse
/// impls will use [`success`] to attach return the Outcome with the newly-parsed value inside.
impl Outcome<()> {
    /// Constructs a new Outcome with empty contents.
    pub fn new() -> Outcome<()> {
        Outcome::Ok(())
    }

    /// Attaches new data to the Outcome and returns the new Outcome wrapped in a [`Result`]. Used
    /// at the end of [`Parse`](syn::Parse) implementations.
    pub fn success<T>(self, value: T) -> Result<Outcome<T>> {
        Ok(match self {
            Outcome::Ok(()) => Outcome::Ok(value),
            Outcome::Continue((), err) => Outcome::Continue(value, err),
            Outcome::NoGenerate(err) => Outcome::NoGenerate(err),
        })
    }
}
```

Then, instead of implementing `syn::Parse` for each AST node type `T`, we
implement it for `Outcome<T>`:

```rust
impl Parse for Outcome<Input> {
    // ...
}

impl Parse for Outcome<Layout> {
    // ...
}

// etc
```

Each `Parse::parse` implementation would construct a new `Outcome` using
`Outcome::new`, and on success would call `Outcome::success` to produce the
`syn::Result<Outcome<T>>` return value.

## Combination functions

This is the part that took too much time to design. `Outcome<()>` needs more
methods than shown above, which combine error signals from other operations into
the overall `Outcome`. For example, it might have:

```rust
impl Outcome<()> {
    /// Used to handle an error from an operation that returns `Result<Node>` directly (rather than
    /// `Result<Outcome<Node>>`). If the operation errored, returns Err (with this Outcome's
    /// accumulated errors prepended); if the operation succeeded, returns the node.
    pub fn chain_result<T>(&mut self, result: Result<T>) -> Result<T> {
        match (self, result) {
            (_, Ok(_)) | (Outcome::Ok(()), Err(_)) => result,
            (Outcome::Continue((), err1) | Outcome::NoGenerate(err1), Err(err2)) => {
                err1.combine(err2);
                // We have to replace the moved-from error with *something*. Since *self should
                // never be used again, we can reset it back to an empty Ok.
                *self = Outcome::Ok(());
                Err(err1)
            },
        }
    }
}
```

which you would use like:

```rust
impl Parse for Outcome<Input> {
    fn parse(input: ParseStream) -> Result<Outcome<Input>> {
        let mut out = Outcome::new();
        // ...
        // This returns early if Punctuated::parse_terminated returns Err(_). In
        // that case, any errors that were already accumulated in `out` are
        // returned as well.
        let layouts = out.chain_result(Punctuated::<Outcome<Layout>, Token![,]>::parse_terminated(input))?;
        // ...
        out.success(Input {
            // ...
        })
    }
}
```

You would also need a `chain_outcome` method to chain a `Result<Outcome<Node>>`
(this would be used in the loop that processes each layout, for instance). In
addition to chaining, you would need methods to add new errors (you would use
separate methods for continuable errors versus no-generate errors).

## Recovering from partially-parsed delimited lists

Recovering from parse errors inside delimited lists (such as the `[]`'s of
register arrays and the `{}`'s of operation lists) *seems* easy, but there's a
catch. `syn` detects if any `ParseBuffer` is only partially consumed, and emits
an "unexpected token" in that case. If you already detected that error and are
recovering from it, you probably don't want `syn`'s error to also be emitted.
Therefore you need to finish consuming the `ParseBuffer`. I haven't found a nice
way to do that, but parsing into a `TokenStream` seems to work and is
infallible:

```rust
/// If a ParseStream is created but only partially consumed, `syn` automatically generates an
/// "unexpected token" error. However, we occasionally parse the contents of delimiters and recover
/// from any errors encountered. This function consumes the remainder of the ParseStream,
/// preventing the "unexpected token" error from being emitted.
// TODO: Verify this is necessary... I think it is if the overall parse succeeds.
fn consume_stream(stream: ParseBuffer) {
    // Parsing a TokenStream should be infallible.
    let _: TokenStream = stream.parse().unwrap();
}
```
