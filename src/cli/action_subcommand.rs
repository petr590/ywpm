use std::collections::HashSet;
use std::fs;
use std::path::Path;

use clap::Subcommand;
use indoc::indoc;
use path_absolutize::Absolutize;

use crate::cli::action_subcommand::ActionSubcommand::*;
use crate::cli::{ArgParseError, ParsedTimePeriod, Settings};
use crate::{arg_parse_error_localized, str_localized, util};

macro_rules! GROUP_NAME {
    () => {
        str_localized!(
            "Group name",
            "Имя группы"
        )
    }
}

#[derive(Debug, PartialEq, Subcommand)]
pub enum ActionSubcommand {
    #[command(
        name = "get",
        about = str_localized!(
            "Get path to current wallpaper",
            "Получить путь к текущим обоям"
        )
    )]
    GetCurrentWallpaper,


    #[command(
        name = "set",
        about = str_localized!(
            "Set specified wallpaper. If folder is specified, random wallpapers from folder are set",
            "Установить указанные обои. Если указана папка, то устанавливаются случайные обои из папки"
        )
    )]
    SetWallpaper {
        #[arg(help = str_localized!(
            "Path to the file/folder",
            "Путь к файлу/папке"
        ))]
        path: String,

        #[command(flatten)]
        settings: Settings,

        #[command(flatten)]
        period: ParsedTimePeriod,
    },


    #[command(
        name = "random",
        about = str_localized!(
            "Set random wallpapers from all in list",
            "Установить случайные обои из всех в списке"
        )
    )]
    SetRandowWallpaper,


    #[command(
        name = "reset",
        about = str_localized!(
            "Reset installed wallpaper",
            "Сбросить установленные обои"
        )
    )]
    ResetWallpaper,


    #[command(
        name = "restore",
        about = str_localized!(
            "Restore previous wallpaper (usually, systemd service automatically passes this parameter to daemon at startup)",
            "Восстановить предыдущие обои (как правило, сервис systemd автоматически передаёт этот параметр демону при запуске)"
        )
    )]
    RestoreWallpaper,


    #[command(
        name = "list",
        about = str_localized!(
            "Get list of all wallpaper paths",
            "Получить список всех путей к обоям"
        )
    )]
    GetNodeList,


    #[command(
        name = "add",
        about = str_localized!(
            "Add a folder/file to list",
            "Добавить папку/файл в список"
        )
    )]
    AddNodes {
        #[arg(
            required = true,
            num_args = 1..,
            help = str_localized!(
                "Mandatory list of paths",
                "Обязательный список путей"
            )
        )]
        paths: Vec<String>,

        #[command(flatten)]
        settings: Settings,

        #[command(flatten)]
        period: ParsedTimePeriod,
    },


    #[command(
        name = "remove",
        about = str_localized!(
            "Remove a folder/file from list (not from disk)",
            "Удалить папку/файл из списка (не с диска)"
        )
    )]
    RemoveNodes {
        #[arg(
            required = true,
            num_args = 1..,
            help = str_localized!(
                "Mandatory list of paths",
                "Обязательный список путей"
            )
        )]
        paths: Vec<String>,
    },


    #[command(
        name = "clear",
        about = str_localized!(
            "Clear all data",
            "Очистить все данные"
        )
    )]
    ClearNodes,


    #[command(
        name = "group-list",
        about = str_localized!(
            "Show list of all groups",
            "Показать список всех групп"
        )
    )]
    GetGroupList,


    #[command(about = str_localized!(
        "Show information and group's composition",
        "Показать информацию и состав группы"
    ))]
    GetGroup {
        #[arg(help = GROUP_NAME!())]
        name: String
    },


    #[command(about = str_localized!(
        "Create new group",
        "Создать новую группу"
    ))]
    NewGroup {
        #[arg(help = GROUP_NAME!())]
        name: String,

        #[arg(num_args = 0..)]
        paths: Vec<String>,
    },


    #[command(about = str_localized!(
        "Set random wallpapers from group",
        "Установить рандомные обои из группы"
    ))]
    SetGroup {
        #[arg(help = GROUP_NAME!())]
        name: String,

        #[command(flatten)] settings: Settings,
        #[command(flatten)] period: ParsedTimePeriod,
    },


    #[command(about = str_localized!(
        "Add files/folders to group",
        "Добавить файлы/папки в группу"
    ))]
    AddToGroup {
        #[arg(help = GROUP_NAME!())]
        name: String,

        #[arg(required = true, num_args = 1..)]
        paths: Vec<String>,
    },


    #[command(about = str_localized!(
        "Remove files/folders from group (not from disk)",
        "Удалить файлы/папки из группы (не с диска)"
    ))]
    RemoveFromGroup {
        #[arg(help = GROUP_NAME!())]
        name: String,

        #[arg(required = true, num_args = 1..)]
        paths: Vec<String>,
    },


    #[command(about = str_localized!(
        "Clear group",
        "Очистить группу"
    ))]
    ClearGroup {
        #[arg(help = GROUP_NAME!())]
        name: String
    },


    #[command(about = str_localized!(
        "Remove group",
        "Удалить группу"
    ))]
    RemoveGroup {
        #[arg(help = str_localized!(
            "Group name",
            "Имя группы"
        ))]
        name: String
    },


    #[command(
        name = "find-non-fitting",
        about = str_localized!(
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
            help = str_localized!(
                "A list of paths for searching. If not specified, list of paths from DB is used",
                "Список путей для поиска. Если не задано, используется список путей из БД"
            )
        )]
        paths: Vec<String>,

        #[arg(
            long,
            help = str_localized!(
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
                *path = canonicalize_path_and_check_is_file(cwd, path)?;
            }

            AddNodes                 { paths, .. } |
            NewGroup                 { paths, .. } |
            AddToGroup               { paths, .. } |
            FindNonFittingWallpapers { paths, .. } => {

                let path_set = paths.iter()
                    .map(|path| canonicalize_path_and_check_is_file(cwd, path))
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
}

fn canonicalize_path_and_check_is_file(cwd: &str, path: &str) -> Result<String, ArgParseError> {
    let abs_path = Path::new(path).absolutize_from(cwd);

    let metadata = fs::metadata(abs_path.as_ref())
            .map_err(|err| ArgParseError::new(err.to_string()))?;

    if metadata.is_file() {
        Ok(abs_path.to_string_lossy().into_owned())
    } else {
        Err(arg_parse_error_localized!(
            "No such file: '{}'",
            "Нет такого файла: '{}'",
            abs_path.to_string_lossy()
        ))
    }
}