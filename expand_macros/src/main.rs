// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

//! This tool is rather simplistic. It does not process `use` statements or understand if the
//! tock-registers crate has been renamed. It simply looks for macro invocations that match the
//! names of known tock-registers macros. The only flexibility it allows is for a leading
//! `tock_registers::` or `::tock_registers::`.
//!
//! If you are using this as part of your build system or code control system, consider moving your
//! tock-register macro invocations into their own files. That way, there's less unrelated code to
//! trip this up. We're open to having the CLI expanded, in case you want to e.g. add flags to
//! write the output into a file.

use clap::{arg, ArgAction::SetTrue, Command};
#[cfg(feature = "prettyplease")]
use prettyplease::unparse;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::{fs::read_to_string, process::exit};
use syn::parse::{ParseStream, Parser};
use syn::{parse_file, Attribute, File, Item, Item::Macro, Result};
use tock_registers_codegen::register_layouts;

fn main() {
    let cli = Command::new(env!("CARGO_PKG_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        // We support this argument whether or not `prettyplease` is enabled, although it does
        // nothing if `prettyplease` is disabled. That allows scripts to always pass -u if they
        // want unformatted output and work with expand_macros binaries compiled either way.
        .arg(arg!(-u --unformatted "Do not pretty-print the expanded code").action(SetTrue))
        .arg(arg!(<FILE>));
    let args = cli.get_matches();

    // Read the input file and parse it into a syn::File;
    let path = args.get_raw("FILE").unwrap().next().unwrap();
    let file = parse_file(&read_to_string(path).unwrap_or_else(|error| {
        eprintln!("Failed to read {path:?}: {error}");
        exit(1);
    }))
    .unwrap_or_else(|error| {
        eprintln!("Parsing failed: {error}");
        exit(1);
    });

    let mut errored = false;
    let mut printer = Printer::new(file.shebang, file.attrs, args.get_flag("unformatted"));
    for item in file.items {
        let Macro(ref mac) = item else {
            printer.push_item(item);
            continue;
        };

        // Extract the macro's name (continuing to the next invocation if we can determine that
        // this is not a tock-registers macro).
        let segments = &mac.mac.path.segments;
        let name = match segments.len() {
            1 if mac.mac.path.leading_colon.is_none() => segments.first().unwrap(),
            2 if segments.first().unwrap().ident == "tock_registers" => segments.get(1).unwrap(),
            _ => {
                printer.push_item(item);
                continue;
            }
        };

        // Check which macro this is, and if it is recognized call that macro's implementation.
        let tokens = &mac.mac.tokens;
        let result = if name.ident == "register_layouts" {
            register_layouts(quote![::tock_registers #tokens])
        } else if name.ident == "mmio32_register_layouts" {
            register_layouts(quote![::tock_registers #![buses(::tock_registers::Mmio32)] #tokens])
        } else if name.ident == "mmio64_register_layouts" {
            register_layouts(quote![::tock_registers #![buses(::tock_registers::Mmio64)] #tokens])
        } else {
            printer.push_item(item);
            continue;
        };

        // Push the result to the printer (as an Item if they successfully parse).
        let out_tokens = result.unwrap_or_else(|e| {
            errored = true;
            e
        });
        match parse_items.parse2(out_tokens.clone()) {
            Ok(items) => items.into_iter().for_each(|item| printer.push_item(item)),
            Err(_) => printer.push_tokens(out_tokens),
        }
    }
    printer.finish();
    if errored {
        exit(1);
    }
}

/// Parses a series of zero or more Items from a ParseStream.
fn parse_items(input: ParseStream) -> Result<Vec<Item>> {
    let mut out = Vec::new();
    while !input.is_empty() {
        out.push(input.parse()?);
    }
    Ok(out)
}

/// Handles formatting the output code (if enabled) and printing it to stdout.
struct Printer {
    /// `Some()` if we're pretty-printing the output, `None` otherwise.
    #[cfg(feature = "prettyplease")]
    file: Option<File>,
}

impl Printer {
    fn new(shebang: Option<String>, attrs: Vec<Attribute>, _unformatted: bool) -> Printer {
        let file = File {
            shebang,
            attrs,
            items: Vec::new(),
        };
        #[cfg(feature = "prettyplease")]
        if !_unformatted {
            return Printer { file: Some(file) };
        }
        print_unformatted(file);
        Printer {
            #[cfg(feature = "prettyplease")]
            file: None,
        }
    }

    fn push_item(&mut self, item: Item) {
        #[cfg(feature = "prettyplease")]
        if let Some(ref mut file) = self.file {
            file.items.push(item);
            return;
        }
        println!("{}", item.into_token_stream());
    }

    /// Pushes a raw token stream to the file. Note that this will disable formatting (as it is
    /// assumed that these tokens could not be parsed as an Item).
    fn push_tokens(&mut self, tokens: TokenStream) {
        #[cfg(feature = "prettyplease")]
        if let Some(file) = self.file.take() {
            print_unformatted(file);
        }
        println!("{tokens}");
    }

    /// Pretty-prints the file, if we are able to format it. If not, this does nothing.
    fn finish(self) {
        #[cfg(feature = "prettyplease")]
        if let Some(file) = self.file {
            println!("{}", unparse(&file));
        }
    }
}

// Prints the contents of the passed File without formatting.
fn print_unformatted(file: File) {
    if let Some(shebang) = file.shebang {
        println!("{shebang}");
    }
    for attr in file.attrs {
        println!("{}", attr.into_token_stream());
    }
    for item in file.items {
        println!("{}", item.into_token_stream());
    }
}
