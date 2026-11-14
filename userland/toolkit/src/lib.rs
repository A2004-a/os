//! NovaOS Apex — Graphical Widget & UI Toolkit (NovaUI)

#![no_std]

extern crate novaos_apex_runtime;

pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
