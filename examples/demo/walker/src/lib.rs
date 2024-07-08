#![no_std]

use sails_rtl::{cell::RefCell, gstd::gservice, prelude::*};

#[derive(Clone)]
pub struct WalkerData {
    x: i32,
    y: i32,
}

impl WalkerData {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Encode, TypeInfo)]
#[codec(crate = sails_rtl::scale_codec)]
#[scale_info(crate = sails_rtl::scale_info)]
enum WalkerEvents {
    Walked { from: (i32, i32), to: (i32, i32) },
}

#[derive(Clone)]
pub struct WalkerService {
    data: &'static RefCell<WalkerData>,
}

impl WalkerService {
    pub fn new(data: &'static RefCell<WalkerData>) -> Self {
        Self { data }
    }
}

#[gservice(events = WalkerEvents)]
impl WalkerService {
    pub fn walk(&mut self, dx: i32, dy: i32) {
        let from = self.position();
        {
            let mut data = self.data.borrow_mut();
            data.x += dx;
            data.y += dy;
        }
        let to = self.position();
        self.notify_on(WalkerEvents::Walked { from, to }).unwrap();
    }

    pub fn position(&self) -> (i32, i32) {
        let data = self.data.borrow();
        (data.x, data.y)
    }
}