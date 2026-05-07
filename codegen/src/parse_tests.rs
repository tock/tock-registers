// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use crate::ast::{Field, FieldDef, Input, PerBusInt, RegisterSpec};
use quote::quote;
use syn::{parse2, parse_quote, TypePath};

// Verifies that outer and inner #[bus] and #[buses] attributes are combined correctly.
#[test]
fn bus() {
    use crate::ast::BusAttr::{Bus, Buses};

    let error = parse2::<Input>(quote![::tock_registers b: a]).unwrap_err();
    assert!(error.to_string().contains("no bus specified"));

    // Shortcuts so the assert_eq!() calls don't line-wrap.
    let mmio32 = || parse_quote![Mmio32];
    let mmio64 = || parse_quote![Mmio64];
    let mmio32null = || parse_quote![Mmio32Nullable];

    let input: Input = parse_quote! {
        ::tock_registers
        #[bus(Mmio32)] a: r,
        #[buses(Mmio32)] b: r,
        #[buses(Mmio32, Mmio64)] c: r,
    };
    assert_eq!(input.layouts[0].bus, Bus(mmio32()));
    assert_eq!(input.layouts[1].bus, Buses(vec![mmio32()]));
    assert_eq!(input.layouts[2].bus, Buses(vec![mmio32(), mmio64()]));

    let input: Input = parse_quote! {
        ::tock_registers #![bus(Mmio32)]
        a: r,
        #[bus(Mmio64)] b: r,
        #[buses(Mmio64)] c: r,
        #[buses(Mmio32Nullable, Mmio64)] d: r,
    };
    assert_eq!(input.layouts[0].bus, Bus(mmio32()));
    assert_eq!(input.layouts[1].bus, Bus(mmio64()));
    assert_eq!(input.layouts[2].bus, Buses(vec![mmio64()]));
    assert_eq!(input.layouts[3].bus, Buses(vec![mmio32null(), mmio64()]));

    let input: Input = parse_quote! {
        ::tock_registers #![buses(Mmio32)]
        a: r,
        #[bus(Mmio64)] b: r,
        #[buses(Mmio64)] c: r,
        #[buses(Mmio32Nullable, Mmio64)] d: r,
    };
    assert_eq!(input.layouts[0].bus, Buses(vec![mmio32()]));
    assert_eq!(input.layouts[1].bus, Bus(mmio64()));
    assert_eq!(input.layouts[2].bus, Buses(vec![mmio64()]));
    assert_eq!(input.layouts[3].bus, Buses(vec![mmio32null(), mmio64()]));

    let input: Input = parse_quote! {
        ::tock_registers #![buses(Mmio32, Mmio32Nullable)]
        a: r,
        #[bus(Mmio64)] b: r,
        #[buses(Mmio64)] c: r,
        #[buses(Mmio32Nullable, Mmio64)] d: r,
    };
    assert_eq!(input.layouts[0].bus, Buses(vec![mmio32(), mmio32null()]));
    assert_eq!(input.layouts[1].bus, Bus(mmio64()));
    assert_eq!(input.layouts[2].bus, Buses(vec![mmio64()]));
    assert_eq!(input.layouts[3].bus, Buses(vec![mmio32null(), mmio64()]));

    let error = parse2::<Input>(quote![::tock_registers #![bus(A)] #![buses(B)] b: a]).unwrap_err();
    assert!(error.to_string().contains("multiple bus attributes"));

    let error = parse2::<Input>(quote![::tock_registers #[buses(A)] #[bus(B)] b: a]).unwrap_err();
    assert!(error.to_string().contains("multiple bus attributes"));
}

#[test]
fn field() {
    let field: Field = parse_quote! {
        ///A
        ///B
        1 => a: b
    };
    assert_eq!(
        field,
        Field {
            offsets: PerBusInt::Single(parse_quote![1]),
            field_def: FieldDef::Register {
                docs: vec![parse_quote![#[doc = r"A"]], parse_quote![#[doc = r"B"]]],
                aliased: false,
                name: parse_quote![a],
                spec: RegisterSpec {
                    element_type: parse_quote![b],
                    array_sizes: vec![],
                    operations: None,
                },
            },
        },
    );

    let error = parse2::<Field>(quote![#[aliased] 1 => _: 2]).unwrap_err();
    assert!(error.to_string().contains("padding cannot be aliased"));

    let field: Field = parse_quote![1 => _: 2];
    assert_eq!(
        field,
        Field {
            offsets: PerBusInt::Single(parse_quote![1]),
            field_def: FieldDef::Padding(Some(PerBusInt::Single(parse_quote![2])))
        },
    );

    let field: Field = parse_quote![#[aliased] [1, 2] => a: u8 { Read }];
    assert_eq!(
        field,
        Field {
            offsets: PerBusInt::Array(vec![parse_quote![1], parse_quote![2]]),
            field_def: FieldDef::Register {
                docs: vec![],
                aliased: true,
                name: parse_quote![a],
                spec: RegisterSpec {
                    element_type: parse_quote![u8],
                    array_sizes: vec![],
                    operations: Some(vec![parse_quote![Read]]),
                },
            },
        },
    );

    let error = parse2::<Field>(quote![#[aliased] #[aliased] 1 => a: status]).unwrap_err();
    assert!(error.to_string().contains("multiple #[aliased] attributes"));

    let error = parse2::<Field>(quote![#[aliased = 3] 1 => a: status]).unwrap_err();
    assert!(error.to_string().contains("cannot have arguments"));

    let error = parse2::<Field>(quote![#[aliased(3)] 1 => a: status]).unwrap_err();
    assert!(error.to_string().contains("cannot have arguments"));
}

#[test]
fn field_def() {
    let field: FieldDef = parse_quote![_: 3];
    assert_eq!(field, FieldDef::Padding(Some(parse_quote![3])));

    let field: FieldDef = parse_quote![_];
    assert_eq!(field, FieldDef::Padding(None));

    let field: FieldDef = parse_quote![a: status];
    assert_eq!(
        field,
        FieldDef::Register {
            docs: vec![],
            aliased: false,
            name: parse_quote![a],
            spec: RegisterSpec {
                element_type: parse_quote![status],
                array_sizes: vec![],
                operations: None,
            }
        }
    );
}

#[test]
fn per_bus_int() {
    let offsets: PerBusInt = parse_quote![0x0];
    assert_eq!(offsets, PerBusInt::Single(parse_quote![0x0]));

    let offsets: PerBusInt = parse_quote!([1, 2, 1]);
    let expected = vec![parse_quote![1], parse_quote![2], parse_quote![1]];
    assert_eq!(offsets, PerBusInt::Array(expected));
}

#[test]
fn register_def() {
    let register: RegisterSpec = parse_quote![: <Foo as Bar>::Associated { Read, Write }];
    let expected_type: TypePath = parse_quote![<Foo as Bar>::Associated];
    assert_eq!(register.element_type, expected_type);
    assert_eq!(register.array_sizes, []);
    let expected_operations = vec![parse_quote![Read], parse_quote![Write]];
    assert_eq!(register.operations, Some(expected_operations));

    let register: RegisterSpec = parse_quote![: status];
    let expected_type: TypePath = parse_quote![status];
    assert_eq!(register.element_type, expected_type);
    assert_eq!(register.array_sizes, []);
    assert_eq!(register.operations, None);

    let register: RegisterSpec = parse_quote![: [[[status; 2]; 3]; 4]];
    let expected_type: TypePath = parse_quote![status];
    assert_eq!(register.element_type, expected_type);
    let expected_sizes = [parse_quote![2], parse_quote![3], parse_quote![4]];
    assert_eq!(register.array_sizes, expected_sizes);
    assert_eq!(register.operations, None);

    let error = parse2::<RegisterSpec>(quote![: <Foo as Bar>::Associated]).unwrap_err();
    assert!(error.to_string().contains("reference must be to a mod"));
}
