// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

//! Input parser.

use crate::ast::{Definition, Field, FieldDef, Input, PerBusInt, RegisterSpec, Value};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::{Brace, Bracket};
use syn::{braced, bracketed, AttrStyle, Attribute, Error, LitInt, Meta, Result, Token, TypePath};

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Input> {
        let tock_registers = input.parse()?;
        // Parse attributes that apply to all definitions.
        let (docs, buses) = definition_attributes(Attribute::parse_inner(input)?)?;
        let definitions: Result<_> = Punctuated::<Definition, Token![,]>::parse_terminated(input)?
            .into_iter()
            .map(|mut definition| {
                // Prepend the global (inner attribute) docs to each Definition's local (outer
                // attribute) docs).
                definition.docs = docs.iter().cloned().chain(definition.docs).collect();
                // Combine the Definition's buses specification with the global buses
                // specification.
                if definition.buses.is_empty() {
                    if buses.is_empty() {
                        return Err(Error::new(definition.name.span(), "no #[buses] specified"));
                    }
                    definition.buses = buses.clone();
                }
                Ok(definition)
            })
            .collect();
        Ok(Input {
            tock_registers,
            definitions: definitions?,
        })
    }
}

impl Parse for Definition {
    fn parse(input: ParseStream) -> Result<Definition> {
        let (docs, buses) = definition_attributes(Attribute::parse_outer(input)?)?;
        Ok(Definition {
            docs,
            buses,
            visibility: input.parse()?,
            name: input.parse()?,
            value: input.parse()?,
        })
    }
}

/// Parses attributes that belong on a Definition. If no `#[buses(...)]` is specified, returns an
/// empty buses. The attributes are returned in order (docs, buses), and are converted into outer
/// attributes.
fn definition_attributes(attributes: Vec<Attribute>) -> Result<(Vec<Attribute>, Vec<TypePath>)> {
    let mut docs = Vec::new();
    let mut buses: Option<Attribute> = None;
    for mut attr in attributes {
        attr.style = AttrStyle::Outer;
        match attr.path() {
            p if p.is_ident("doc") => docs.push(attr),
            p if p.is_ident("buses") => {
                if let Some(prev_buses) = buses {
                    let mut error = Error::new_spanned(attr, "multiple #[buses()] attributes");
                    error.combine(Error::new_spanned(
                        prev_buses,
                        "note: #[buses()] already specified here",
                    ));
                    return Err(error);
                }
                buses = Some(attr);
            }
            p => return Err(Error::new(p.span(), "unknown attribute")),
        }
    }
    let Some(buses) = buses else {
        return Ok((docs, Vec::new()));
    };
    let buses_vec: Vec<_> = buses
        .parse_args_with(Punctuated::<_, Token![,]>::parse_terminated)?
        .into_iter()
        .collect();
    if buses_vec.is_empty() {
        return Err(Error::new_spanned(buses, "buses list cannot be empty"));
    }
    Ok((docs, buses_vec))
}

impl Parse for Value {
    fn parse(input: ParseStream) -> Result<Value> {
        // Distinguish between a single register and block by looking at the first token, which
        // should be either : (single) or { (block).
        if input.peek(Token![:]) {
            return input.parse().map(Value::Single);
        }
        if !input.peek(Brace) {
            return Err(input.error("expected one of: `:`, `{`"));
        }
        let fields;
        braced!(fields in input);
        let fields: Vec<Field> = Punctuated::<_, Token![,]>::parse_terminated(&fields)?
            .into_iter()
            .collect();
        for field in fields.iter().rev() {
            match field.field_def {
                FieldDef::Padding(None) => {
                    return Err(Error::new(
                        field.offsets[0].span(),
                        "last non-aliased field cannot be padding without a size",
                    ))
                }
                FieldDef::Register { aliased: true, .. } => continue,
                FieldDef::Padding(Some(_)) | FieldDef::Register { aliased: false, .. } => break,
            }
        }
        Ok(Value::Block(fields))
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Field> {
        let mut aliased_attr: Option<Attribute> = None;
        let mut doc_attrs = Vec::new();
        for attr in Attribute::parse_outer(input)? {
            match attr.path() {
                p if p.is_ident("doc") => doc_attrs.push(attr),
                p if p.is_ident("aliased") => {
                    if let Some(prev) = aliased_attr {
                        let mut error = Error::new_spanned(attr, "multiple #[aliased] attributes");
                        error.combine(Error::new_spanned(
                            prev,
                            "note: aliased already specified here",
                        ));
                        return Err(error);
                    }
                    let Meta::Path(_) = attr.meta else {
                        return Err(Error::new_spanned(attr, "#[aliased] cannot have arguments"));
                    };
                    aliased_attr = Some(attr);
                }
                p => return Err(Error::new(p.span(), "unknown attribute")),
            }
        }
        let offsets = input.parse()?;
        input.parse::<Token![=>]>()?;
        let mut field_def = input.parse()?;
        match field_def {
            FieldDef::Padding(_) => {
                if let Some(last) = doc_attrs.last() {
                    return Err(Error::new(last.span(), "padding cannot have doc comments"));
                }
                if let Some(aliased) = aliased_attr {
                    return Err(Error::new_spanned(aliased, "padding cannot be aliased"));
                }
            }
            FieldDef::Register {
                ref mut docs,
                ref mut aliased,
                ..
            } => {
                *docs = doc_attrs;
                *aliased = aliased_attr.is_some();
            }
        }
        Ok(Field { offsets, field_def })
    }
}

impl Parse for FieldDef {
    fn parse(input: ParseStream) -> Result<FieldDef> {
        if input.peek(Token![_]) {
            // The underscore tells us this field is padding.
            input.parse::<Token![_]>()?;
            if !input.peek(Token![:]) {
                return Ok(FieldDef::Padding(None));
            }
            input.parse::<Token![:]>()?;
            return Ok(FieldDef::Padding(Some(input.parse()?)));
        }
        Ok(FieldDef::Register {
            docs: Vec::new(),
            aliased: false,
            name: input.parse()?,
            definition: input.parse()?,
        })
    }
}

impl Parse for PerBusInt {
    fn parse(input: ParseStream) -> Result<PerBusInt> {
        if !input.peek(Bracket) {
            return input.parse().map(PerBusInt::Single);
        }
        let contents;
        bracketed!(contents in input);
        let ints = Punctuated::<_, Token![,]>::parse_terminated(&contents)?;
        if ints.is_empty() {
            return Err(Error::new(contents.span(), "offset list cannot be empty"));
        }
        Ok(PerBusInt::Array(ints.into_iter().collect()))
    }
}

impl Parse for RegisterSpec {
    fn parse(input: ParseStream) -> Result<RegisterSpec> {
        input.parse::<Token![:]>()?;
        // Recursive function to parse the type specification (because syn makes it hard to consume
        // individual bracket tokens).
        fn parse_type(input: ParseStream, array_sizes: &mut Vec<LitInt>) -> Result<TypePath> {
            if !input.peek(Bracket) {
                return input.parse();
            }
            let inner;
            bracketed!(inner in input);
            let out = parse_type(&inner, array_sizes)?;
            inner.parse::<Token![;]>()?;
            array_sizes.push(inner.parse()?);
            Ok(out)
        }
        let mut array_sizes = Vec::new();
        let element_type = parse_type(input, &mut array_sizes)?;
        let operations = if input.peek(Brace) {
            let ops;
            braced!(ops in input);
            let ops = Punctuated::<_, Token![,]>::parse_terminated(&ops)?;
            Some(ops.into_iter().collect())
        } else if element_type.qself.is_some() {
            let err = Error::new_spanned(element_type, "register reference must be to a module");
            return Err(err);
        } else {
            None
        };
        Ok(RegisterSpec {
            element_type,
            array_sizes,
            operations,
        })
    }
}
