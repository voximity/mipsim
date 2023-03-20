use std::collections::HashMap;

use egui::{Key, KeyboardShortcut, Modifiers};
use lazy_static::lazy_static;

use crate::{
    app::App,
    simulator::{ProcMessage, ADDR_STATIC},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    File,
    Edit,
    View,
    Run,
}

impl Category {
    pub fn name(&self) -> &'static str {
        match self {
            Self::File => "File",
            Self::Edit => "Edit",
            Self::View => "View",
            Self::Run => "Run",
        }
    }
}

pub static CATEGORIES: &[Category] = &[
    Category::File,
    Category::Edit,
    Category::View,
    Category::Run,
];

pub struct Command {
    pub name: &'static str,
    pub category: Category,
    pub keybind: Option<KeyboardShortcut>,
    pub action: fn(CommandCtx<'_>),
}

pub struct CommandCtx<'a> {
    pub app: &'a mut App,
    pub ctx: &'a egui::Context,
    pub frame: &'a mut eframe::Frame,
}

macro_rules! add_modifiers {
    ($mod:ident, $($other:ident),*) => {
        Modifiers::$mod.plus(add_modifiers!($($other),*))
    };
    ($mod:ident) => {
        Modifiers::$mod
    };
}

macro_rules! keyboard_shortcut {
    (+ None) => {
        None
    };
    ($( $modifier: ident ),* + $key:ident ) => {
        Some(KeyboardShortcut {
            modifiers: add_modifiers!($($modifier),*),
            key: Key::$key
        })
    };
}

macro_rules! commands {
    { $( $category:ident / $name:literal ($( $modifier:ident ),* + $key:ident ) => $action_name:ident $action:item ),*, } => {
        pub static COMMANDS: &[Command] = &[$(
            Command {
                name: $name,
                category: Category::$category,
                keybind: keyboard_shortcut!($($modifier),* + $key),
                action: $action_name,
            },
        )*];

        $($action)*
    }
}

commands! {
    File / "New File" (CTRL + N) => command_new_file
        fn command_new_file(ctx: CommandCtx<'_>) {
            ctx.app.set_file(None, ctx.frame);
        },

    File / "Open File" (CTRL + O) => command_open_file
        fn command_open_file(ctx: CommandCtx<'_>) {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("MIPS Assembly Files", &["s"])
                .pick_file()
            {
                ctx.app.load_file(path, ctx.frame).expect("failed to load file");
            }
        },

    File / "Save File" (CTRL + S) => command_save_file
        fn command_save_file(ctx: CommandCtx<'_>) {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("MIPS Assembly Files", &["s"])
                .pick_file()
            {
                ctx.app.load_file(path, ctx.frame).expect("failed to save file");
            }
        },

    File / "Save File As" (CTRL, SHIFT + S) => command_save_file_as
        fn command_save_file_as(ctx: CommandCtx<'_>) {
            ctx.app.save_file(true, ctx.frame).expect("failed to save file");
        },

    Run / "Assemble" (+ None) => command_assemble
        fn command_assemble(ctx: CommandCtx<'_>) {
            ctx.app.proc_tx.send(ProcMessage::Load(ctx.app.body.clone())).unwrap();
        },

    Run / "Reset" (CTRL, SHIFT + R) => command_reset
        fn command_reset(ctx: CommandCtx<'_>) {
            ctx.app.proc.pc_lines = None;
            ctx.app.output.io.reset();
            ctx.app.memory.offset = ADDR_STATIC;
            ctx.app.proc_tx.send(ProcMessage::Reset).unwrap();
        },

    Run / "Step" (CTRL + Space) => command_step
        fn command_step(ctx: CommandCtx<'_>) {
            ctx.app.proc_tx.send(ProcMessage::Step).unwrap();
        },
}

lazy_static! {
    pub static ref COMMAND_CATEGORIES: HashMap<Category, Vec<&'static Command>> = {
        let mut map: HashMap<Category, Vec<&'static Command>> =
            CATEGORIES.iter().map(|c| (*c, vec![])).collect();
        COMMANDS
            .iter()
            .for_each(|c| map.get_mut(&c.category).unwrap().push(c));
        map
    };
}
