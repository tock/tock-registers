// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

//! Tests passing a RegisterSender between threads and using it to access a single peripheral.

use std::{ptr::NonNull, sync::mpsc::channel, thread::spawn};
use tock_registers::{mmio64_register_layouts, Mmio64, Read, RegisterSender, Write};

mmio64_register_layouts! {
    counter {
        0 => ctrl: u8 { Read, Write },
    }
}

#[test]
fn two_threads() {
    use counter::Interface;
    // This test spawns a second thread. It constructs the RegisterSender on the main thread, sends
    // it to the second thread through this channel, and then returns the RegisterSender to the
    // main thread by returning it from the second thread.
    let (sender, receiver) = channel();
    let join = spawn(move || {
        let register_sender: RegisterSender<counter::Real<_>> = receiver.recv().unwrap();
        assert_eq!(register_sender.borrow().ctrl().get(), 2);
        register_sender.borrow().ctrl().set(3);
        register_sender
    });
    let mut peripheral: u8 = 1;
    let mmio = Mmio64::new(NonNull::from(&mut peripheral).cast());
    let register_sender = unsafe { RegisterSender::<counter::Real<_>>::new(mmio) };
    // Use the RegisterSender from this thread.
    assert_eq!(register_sender.borrow().ctrl().get(), 1);
    register_sender.borrow().ctrl().set(2);
    // Send the RegisterSender to the second thread, which will use it to increment the byte.
    sender.send(register_sender).unwrap();
    // Join the second thread, retrieving the RegisterSender.
    let register_sender = join.join().unwrap();
    assert_eq!(register_sender.borrow().ctrl().get(), 3);
    register_sender.borrow().ctrl().set(4);
}
