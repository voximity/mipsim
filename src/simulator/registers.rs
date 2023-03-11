use std::mem::transmute;

use egui_extras::{Column, TableBuilder};

use super::{ADDR_HEAP, ADDR_STACK_TOP};

#[derive(Debug)]
pub struct Registers {
    pub data: [Register; 32],
}

impl Default for Registers {
    fn default() -> Self {
        let mut data = [Register(0); 32];
        data[28] = Register(unsafe { transmute(ADDR_HEAP as u32) });
        data[29] = Register(unsafe { transmute(ADDR_STACK_TOP as u32) });
        Self { data }
    }
}

impl Registers {
    #[rustfmt::skip]
    pub const fn name(i: usize) -> &'static str {
        match i {
            0 => "zero",
            1 => "at",
            2 => "v0", 3 => "v1",
            4 => "a0", 5 => "a1", 6 => "a2", 7 => "a3",
            8 => "t0", 9 => "t1", 10 => "t2", 11 => "t3",
            12 => "t4", 13 => "t5", 14 => "t6", 15 => "t7",
            16 => "s0", 17 => "s1", 18 => "s2", 19 => "s3",
            20 => "s4", 21 => "s5", 22 => "s6", 23 => "s7",
            24 => "t8", 25 => "t9",
            26 => "k0", 27 => "k1",
            28 => "gp",
            29 => "sp",
            30 => "fp",
            31 => "ra",
            _ => panic!("invalid register index"),
        }
    }

    #[rustfmt::skip]
    pub fn index(s: &str) -> Option<usize> {
        Some(match s {
            "zero" => 0,
            "at" => 1,
            "v0" => 2, "v1" => 3,
            "a0" => 4, "a1" => 5, "a2" => 6, "a3" => 7,
            "t0" => 8, "t1" => 9, "t2" => 10, "t3" => 11,
            "t4" => 12, "t5" => 13, "t6" => 14, "t7" => 15,
            "s0" => 16, "s1" => 17, "s2" => 18, "s3" => 19,
            "s4" => 20, "s5" => 21, "s6" => 22, "s7" => 23,
            "t8" => 24, "t9" => 25,
            "k0" => 26, "k1" => 27,
            "gp" => 28,
            "sp" => 29,
            "fp" => 30,
            "ra" => 31,
            _ => s.parse().ok()?
        })
    }

    pub fn set_i32(&mut self, index: u8, value: i32) {
        self.data[index as usize] = Register(value);
    }

    pub fn set_u32(&mut self, index: u8, value: u32) {
        self.data[index as usize] = unsafe { transmute(value) };
    }

    pub fn get_i32(&self, index: u8) -> i32 {
        self.data[index as usize].0
    }

    pub fn get_u32(&self, index: u8) -> u32 {
        self.data[index as usize].to_u32()
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        TableBuilder::new(ui)
            .column(Column::auto().at_least(60.0).resizable(false))
            .column(Column::auto().at_least(30.0).resizable(false))
            .column(Column::remainder().resizable(false))
            .striped(true)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Register");
                });
                header.col(|ui| {
                    ui.strong("Num.");
                });
                header.col(|ui| {
                    ui.strong("Value");
                });
            })
            .body(|body| {
                body.rows(14.0, 32, |i, mut row| {
                    row.col(|ui| {
                        ui.monospace(format!("${}", Self::name(i)));
                    });
                    row.col(|ui| {
                        ui.monospace(format!("{i}"));
                    });
                    row.col(|ui| {
                        ui.label(format!("0x{:08x}", self.data[i].0));
                    });
                })
            })
    }
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Register(i32);

impl Register {
    pub fn to_u32(self) -> u32 {
        // SAFETY: i32 and u32 share a size
        unsafe { transmute(self.0) }
    }
}
