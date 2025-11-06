use crate::theme::Theme;
use gpui::*;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Clone, Copy)]
pub enum IconSize {
    Small,   // 12px
    Medium,  // 16px
    Large,   // 24px
}

impl IconSize {
    fn to_px(&self) -> Pixels {
        match self {
            IconSize::Small => px(12.0),
            IconSize::Medium => px(16.0),
            IconSize::Large => px(24.0),
        }
    }
}

pub enum IconType {
    File,
    Folder,
    FolderOpen,
    Save,
    Open,
    Close,
    Settings,
    Search,
    Error,
    Warning,
    Info,
    Success,
    ChevronRight,
    ChevronDown,
}

impl IconType {
    fn to_emoji(&self) -> &'static str {
        match self {
            IconType::File => "📄",
            IconType::Folder => "📁",
            IconType::FolderOpen => "📂",
            IconType::Save => "💾",
            IconType::Open => "📂",
            IconType::Close => "✕",
            IconType::Settings => "⚙️",
            IconType::Search => "🔍",
            IconType::Error => "❌",
            IconType::Warning => "⚠️",
            IconType::Info => "ℹ️",
            IconType::Success => "✓",
            IconType::ChevronRight => "›",
            IconType::ChevronDown => "⌄",
        }
    }
}

pub struct Icon {
    icon_type: IconType,
    size: IconSize,
    theme: Arc<RwLock<Theme>>,
}

impl Icon {
    pub fn new(icon_type: IconType, size: IconSize, theme: Arc<RwLock<Theme>>) -> Self {
        Self {
            icon_type,
            size,
            theme,
        }
    }
}

impl Render for Icon {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.read();
        let color = theme.parse_color(&theme.foreground.editor);

        div()
            .size(self.size.to_px())
            .flex()
            .items_center()
            .justify_center()
            .text_color(color)
            .child(self.icon_type.to_emoji())
    }
}

