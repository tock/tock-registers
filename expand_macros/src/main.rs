// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use clap::{arg, Command};
use prettyplease::unparse;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::{fs::read_to_string, process::exit};
use syn::parse::{ParseStream, Parser};
use syn::{parse_file, Attribute, File, Item, Item::Macro, Result};
use tock_registers_codegen::{register_map, Env::External};

fn main() {
    let cli = Command::new(env!("CARGO_PKG_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
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
    let mut printer = Printer::new(file.shebang, file.attrs);
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
        let result = if name.ident == "register_map" {
            register_map(quote![::tock_registers #tokens], External)
        } else if name.ident == "mmio32_register_map" {
            register_map(
                quote![::tock_registers #![bus(::tock_registers::Mmio32)] #tokens],
                External,
            )
        } else if name.ident == "mmio64_register_map" {
            register_map(
                quote![::tock_registers #![bus(::tock_registers::Mmio64)] #tokens],
                External,
            )
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
    /// Pretty-printing is not always possible: it can fail if syn is unable to parse an Item. In
    /// that case, this Printer will switch to dumping unformatted tokens as they are pushed rather
    /// than collecting them into a File. If that happens, this will be switched to `None`.
    file: Option<File>,
}

impl Printer {
    fn new(shebang: Option<String>, attrs: Vec<Attribute>) -> Printer {
        Printer {
            file: Some(File {
                shebang,
                attrs,
                items: Vec::new(),
            }),
        }
    }

    fn push_item(&mut self, item: Item) {
        match &mut self.file {
            Some(file) => file.items.push(item),
            None => println!("{}", item.into_token_stream()),
        }
    }

    /// Pushes a raw token stream to the file. Note that this will disable formatting (as it is
    /// assumed that these tokens could not be parsed as an Item).
    fn push_tokens(&mut self, tokens: TokenStream) {
        if let Some(file) = self.file.take() {
            print_unformatted(file);
        }
        println!("{tokens}");
    }

    /// Pretty-prints the file, if we are able to format it. If not, this does nothing.
    fn finish(self) {
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
