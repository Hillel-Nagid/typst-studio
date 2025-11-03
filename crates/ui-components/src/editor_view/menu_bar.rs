//! Menu bar component for the editor
//!
//! Phase 3.1: Editor View Component Hierarchy - Menu System

use gpui::*;

/// Menu item definition
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub action: String,
}

impl MenuItem {
    pub fn new(label: &str, action: &str) -> Self {
        Self {
            label: label.to_string(),
            action: action.to_string(),
        }
    }
}

/// Menu definition (top-level menu like "File", "Edit")
#[derive(Debug, Clone)]
pub struct Menu {
    pub title: String,
    pub items: Vec<MenuItem>,
}

impl Menu {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            items: Vec::new(),
        }
    }

    pub fn add_item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }
}

/// Menu bar component
pub struct MenuBar {
    pub menus: Vec<Menu>,
}

impl MenuBar {
    pub fn new() -> Self {
        Self {
            menus: vec![
                Self::file_menu(),
                Self::edit_menu(),
                Self::view_menu(),
                Self::compile_menu(),
                Self::help_menu()
            ],
        }
    }

    fn file_menu() -> Menu {
        Menu::new("File")
            .add_item(MenuItem::new("New", "file.new"))
            .add_item(MenuItem::new("Open", "file.open"))
            .add_item(MenuItem::new("Save", "file.save"))
            .add_item(MenuItem::new("Save As", "file.save_as"))
            .add_item(MenuItem::new("Close", "file.close"))
            .add_item(MenuItem::new("Exit", "file.exit"))
    }

    fn edit_menu() -> Menu {
        Menu::new("Edit")
            .add_item(MenuItem::new("Undo", "edit.undo"))
            .add_item(MenuItem::new("Redo", "edit.redo"))
            .add_item(MenuItem::new("Cut", "edit.cut"))
            .add_item(MenuItem::new("Copy", "edit.copy"))
            .add_item(MenuItem::new("Paste", "edit.paste"))
            .add_item(MenuItem::new("Find", "edit.find"))
            .add_item(MenuItem::new("Replace", "edit.replace"))
    }

    fn view_menu() -> Menu {
        Menu::new("View")
            .add_item(MenuItem::new("Toggle Sidebar", "view.toggle_sidebar"))
            .add_item(MenuItem::new("Toggle Preview", "view.toggle_preview"))
            .add_item(MenuItem::new("Zoom In", "view.zoom_in"))
            .add_item(MenuItem::new("Zoom Out", "view.zoom_out"))
            .add_item(MenuItem::new("Toggle Theme", "view.toggle_theme"))
    }

    fn compile_menu() -> Menu {
        Menu::new("Compile")
            .add_item(MenuItem::new("Compile Document", "compile.compile"))
            .add_item(MenuItem::new("Export PDF", "compile.export_pdf"))
            .add_item(MenuItem::new("Export PNG", "compile.export_png"))
    }

    fn help_menu() -> Menu {
        Menu::new("Help")
            .add_item(MenuItem::new("Documentation", "help.docs"))
            .add_item(MenuItem::new("Keyboard Shortcuts", "help.shortcuts"))
            .add_item(MenuItem::new("About", "help.about"))
    }
}

impl Default for MenuBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for MenuBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .gap(px(0.0))
            .children(
                self.menus.iter().map(|menu| {
                    div()
                        .px(px(12.0))
                        .py(px(8.0))
                        .child(menu.title.clone())
                        .text_color(rgb(0xcccccc))
                        .text_size(px(14.0))
                        .hover(|style| style.bg(rgb(0x3e3e42)))
                })
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_bar_creation() {
        let menu_bar = MenuBar::new();
        assert_eq!(menu_bar.menus.len(), 5);
        assert_eq!(menu_bar.menus[0].title, "File");
        assert_eq!(menu_bar.menus[1].title, "Edit");
        assert_eq!(menu_bar.menus[2].title, "View");
        assert_eq!(menu_bar.menus[3].title, "Compile");
        assert_eq!(menu_bar.menus[4].title, "Help");
    }

    #[test]
    fn test_file_menu_items() {
        let menu_bar = MenuBar::new();
        let file_menu = &menu_bar.menus[0];
        assert!(file_menu.items.len() > 0);
        assert_eq!(file_menu.items[0].label, "New");
    }
}
