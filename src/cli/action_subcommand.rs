use std::collections::HashSet;
use std::fs;
use std::path::Path;

use clap::{Subcommand, ValueHint};
use indoc::indoc;
use path_absolutize::Absolutize;

use crate::cli::action_subcommand::ActionSubcommand::*;
use crate::cli::{ArgParseError, ParsedTimePeriod, Settings};
use crate::{arg_parse_error_localized, localized, util};

macro_rules! GROUP_NAME {
    () => {
        localized!(
            "Group name",
            "Имя группы"
        )
    }
}

macro_rules! MANDATORY_PATHS {
    () => {
        localized!(
            "Mandatory list of paths",
            "Обязательный список путей"
        )
    };
}

macro_rules! OPTIONAL_PATHS {
    () => {
        localized!(
            "Optional list of paths",
            "Необязательный список путей"
        )
    };
}

#[derive(Debug, PartialEq, Subcommand)]
pub enum ActionSubcommand {
    #[command(
        name = "get",
        about = localized!(
            "Get path to current wallpaper",
            "Получить путь к текущим обоям"
        )
    )]
    GetCurrentWallpaper,


    #[command(
        name = "set",
        about = localized!(
            "Set specified wallpaper. If folder is specified, random wallpapers from folder are set",
            "Установить указанные обои. Если указана папка, то устанавливаются случайные обои из папки"
        )
    )]
    SetWallpaper {
        #[arg(
            value_hint = ValueHint::AnyPath,
            help = localized!(
                "Path to the file/folder",
                "Путь к файлу/папке"
            )
        )]
        path: String,

        #[command(flatten)] settings: Settings,
        #[command(flatten)] period: ParsedTimePeriod,
    },


    #[command(
        name = "random",
        about = localized!(
            "Set random wallpapers from all in list",
            "Установить случайные обои из всех в списке"
        )
    )]
    SetRandowWallpaper,


    #[command(
        name = "reset",
        about = localized!(
            "Reset installed wallpaper",
            "Сбросить установленные обои"
        )
    )]
    ResetWallpaper,


    #[command(
        name = "restore",
        about = localized!(
            "Restore previous wallpaper (usually, systemd service automatically passes this parameter to daemon at startup)",
            "Восстановить предыдущие обои (как правило, сервис systemd автоматически передаёт этот параметр демону при запуске)"
        )
    )]
    RestoreWallpaper,


    #[command(
        name = "list",
        about = localized!(
            "Get list of all wallpaper paths",
            "Получить список всех путей к обоям"
        )
    )]
    GetNodeList,


    #[command(
        name = "add",
        about = localized!(
            "Add a folder/file to list",
            "Добавить папку/файл в список"
        )
    )]
    AddNodes {
        #[arg(
            required = true,
            value_hint = ValueHint::AnyPath,
            help = MANDATORY_PATHS!()
        )]
        paths: Vec<String>,

        #[command(flatten)] settings: Settings,
        #[command(flatten)] period: ParsedTimePeriod,
    },


    #[command(
        name = "update",
        about = localized!(
            "Update folder/file settings. If not specified, it is updated for current",
            "Обновить настройки папки/файла. Если не указано, обновляется для текущего"
        )
    )]
    UpdateNodes {
        #[arg(
            value_hint = ValueHint::AnyPath,
            help = OPTIONAL_PATHS!()
        )]
        paths: Vec<String>,

        #[command(flatten)] settings: Settings,
        #[command(flatten)] period: ParsedTimePeriod,
    },


    #[command(
        name = "remove",
        about = localized!(
            "Remove a folder/file from list (not from disk)",
            "Удалить папку/файл из списка (не с диска)"
        )
    )]
    RemoveNodes {
        #[arg(
            required = true,
            value_hint = ValueHint::AnyPath,
            help = MANDATORY_PATHS!()
        )]
        paths: Vec<String>,
    },


    #[command(
        name = "clear",
        about = localized!(
            "Clear all data",
            "Очистить все данные"
        )
    )]
    ClearNodes,


    #[command(
        name = "group-list",
        about = localized!(
            "Show list of all groups",
            "Показать список всех групп"
        )
    )]
    GetGroupList,


    #[command(about = localized!(
        "Show information and group's composition",
        "Показать информацию и состав группы"
    ))]
    GetGroup {
        #[arg(help = GROUP_NAME!())]
        name: String
    },


    #[command(about = localized!(
        "Create new group",
        "Создать новую группу"
    ))]
    NewGroup {
        #[arg(help = GROUP_NAME!())]
        name: String,

        #[arg(
            value_hint = ValueHint::AnyPath,
            help = OPTIONAL_PATHS!()
        )]
        paths: Vec<String>,
    },


    #[command(about = localized!(
        "Set random wallpapers from group",
        "Установить рандомные обои из группы"
    ))]
    SetGroup {
        #[arg(help = GROUP_NAME!())]
        name: String,

        #[command(flatten)] settings: Settings,
        #[command(flatten)] period: ParsedTimePeriod,
    },


    #[command(about = localized!(
        "Add files/folders to group",
        "Добавить файлы/папки в группу"
    ))]
    AddToGroup {
        #[arg(help = GROUP_NAME!())]
        name: String,

        #[arg(
            required = true,
            value_hint = ValueHint::AnyPath,
            help = MANDATORY_PATHS!()
        )]
        paths: Vec<String>,
    },


    #[command(about = localized!(
        "Remove files/folders from group (not from disk)",
        "Удалить файлы/папки из группы (не с диска)"
    ))]
    RemoveFromGroup {
        #[arg(help = GROUP_NAME!())]
        name: String,

        #[arg(
            required = true,
            value_hint = ValueHint::AnyPath,
            help = MANDATORY_PATHS!()
        )]
        paths: Vec<String>,
    },


    #[command(about = localized!(
        "Clear group",
        "Очистить группу"
    ))]
    ClearGroup {
        #[arg(help = GROUP_NAME!())]
        name: String
    },


    #[command(about = localized!(
        "Remove group",
        "Удалить группу"
    ))]
    RemoveGroup {
        #[arg(help = localized!(
            "Group name",
            "Имя группы"
        ))]
        name: String
    },


    #[command(
        name = "find-non-fitting",
        about = localized!(
            indoc! {"
                Find all images and videos whose aspect ratio differs from monitor and for which --mode is
                not specified. For videos, it also checks for pixel-by-pixel resolution matching, as real-time
                video scaling is expensive operation.
            "},
            indoc! {"
                Найти все изображения и видео, у которых соотношение сторон отличаеся от монитора и для
                которых не задан --mode. Для видео также проверяет попиксельное совпадение разрешения, так как
                масштабирование видео в реальном времени - недешёвая операция.
            "}
        )
    )]
    FindNonFittingWallpapers {
        #[arg(
            num_args = 0..,
            value_hint = ValueHint::AnyPath,
            help = localized!(
                "A list of paths for searching. If not specified, list of paths from DB is used",
                "Список путей для поиска. Если не задано, используется список путей из БД"
            )
        )]
        paths: Vec<String>,

        #[arg(
            long,
            help = localized!(
                "Display ID if there is more than one display on the computer",
                "ID дисплея в случае, если на компьютере более одного дисплея"
            )
        )]
        display_id: Option<u32>,
    },
}

impl ActionSubcommand {

    pub(super) fn canonicalize_paths(&mut self, cwd: &str) -> Result<(), ArgParseError> {
        match self {
            SetWallpaper { path, .. } => {
                *path = canonicalize_path_and_check_exists(cwd, path)?;
            }

            AddNodes                 { paths, .. } |
            NewGroup                 { paths, .. } |
            AddToGroup               { paths, .. } |
            FindNonFittingWallpapers { paths, .. } => {

                let path_set = paths.iter()
                    .map(|path| canonicalize_path_and_check_exists(cwd, path))
                    .collect::<Result<HashSet<String>, ArgParseError>>()?;

                *paths = path_set.into_iter().collect();
            }

            RemoveNodes     { paths, .. } |
            RemoveFromGroup { paths, .. } => {

                let path_set = paths.iter()
                    .map(|path| util::canonicalize_path(cwd, path))
                    .collect::<HashSet<String>>();

                *paths = path_set.into_iter().collect();
            }

            _ => {}
        }

        Ok(())
    }

    pub fn changes_state(&self) -> bool {
        match self {
            SetWallpaper             { .. } |
            SetRandowWallpaper       { .. } |
            ResetWallpaper           { .. } |
            AddNodes                 { .. } |
            RemoveNodes              { .. } |
            ClearNodes               { .. } |
            NewGroup                 { .. } |
            SetGroup                 { .. } |
            AddToGroup               { .. } |
            RemoveFromGroup          { .. } |
            ClearGroup               { .. } |
            RemoveGroup              { .. } |
            FindNonFittingWallpapers { .. } => true,

            _ => false,
        }
    }
}

fn canonicalize_path_and_check_exists(cwd: &str, path: &str) -> Result<String, ArgParseError> {
    let abs_path = Path::new(path).absolutize_from(cwd);

    let metadata = fs::metadata(abs_path.as_ref())
            .map_err(|err| ArgParseError::new(err.to_string()))?;

    if metadata.is_file() || metadata.is_dir() {
        Ok(abs_path.to_string_lossy().into_owned())
    } else {
        Err(arg_parse_error_localized!(
            "No such file or directory: '{}'",
            "Нет такого файла или каталога: '{}'",
            abs_path.to_string_lossy()
        ))
    }
}
