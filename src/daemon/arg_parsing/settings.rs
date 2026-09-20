use std::cell::RefMut;
use std::str::FromStr;

use clap::Args;
use indoc::indoc;

use crate::daemon::state::{DisplayMode, WallpaperNode};
use crate::str_localized;

/// Настройки для WallpaperNode. Каждое значение опционально, так как юзер может задать или не задать определённую настройку.
#[derive(Debug, PartialEq, Clone, Args)]
pub struct Settings {
    #[arg(
        short, long,
        value_parser = DisplayMode::from_str,
        help = str_localized!(
            indoc! {"
                Wallpaper display mode. You can set vertical alignment
                (top/center/bottom), horizontal alignment (left/center/right),
                and stretch mode:
                - cover - crops edges, maintains aspect ratio
                - contain - leaves white space around edges, maintains aspect ratio
                - stretch - does not maintain aspect ratio, stretching the image
                Default: 'center cover'
            "},
            indoc! {"
                Режим отображения обоев. Можно задать выравнивание по вертикали
                (top/center/bottom), по горизонтали (left/center/right) и режим
                растягивания:
                - cover - Обрезает края, сохраняет пропорции
                - contain - Оставляет пустое место по краям, сохраняет пропорции
                - stretch - Не сохраняет пропорции, растягивая картинку
                По умолчанию: 'center cover'
            "}
        ) 
    )]
    pub mode: Option<DisplayMode>,

    #[arg(
        short, long,
        help = str_localized!(
            "The recursive search level for files in a folder. Range: 1..65535. Default: 1",
            "Уровень рекурсивного поиска файлов в папке. Диапазон: 1..65535. По умолчанию: 1"
        )
    )]
    pub recursive_level: Option<u16>,
}

impl Settings {
    pub const fn new() -> Self {
        Self {
            mode: None,
            recursive_level: None,
        }
    }

    pub fn is_some(&self) -> bool {
        self.mode.is_some() || self.recursive_level.is_some()
    }

    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    pub fn update_node(&self, node: &mut RefMut<WallpaperNode>) {
        if let Some(mode) = &self.mode {
            node.mode = mode.clone();
        }

        if let Some(recursive_level) = self.recursive_level {
            node.recursive_level = recursive_level;
        }
    }
}
