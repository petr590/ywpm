use std::str::FromStr;

use crate::{arg_parse_error_localized, arg_parse_error_localized_with_usage};
use crate::daemon::action::Action;
use crate::daemon::arg_parsing::action::{ArgParseAction, UniqueOption};
use crate::daemon::arg_parsing::{ArgParseError, time_parser};
use crate::daemon::state::DisplayMode;

#[macro_export]
macro_rules! GET_HELP_MESSAGE {
    ($cmd:expr) => {
        if *crate::util::IS_RU {
            $crate::indoc::formatdoc! ("
                Утилита управления обоями рабочего стола
                ИСПОЛЬЗОВАНИЕ:
                    {} <ДЕЙСТВИЕ> [ОПЦИИ] [АРГУМЕНТЫ]

                ДЕЙСТВИЯ (Обои):
                    list                    Получить список всех путей к обоям
                    get                     Получить путь к текущим обоям
                    set <ПУТЬ>              Установить указанные обои. Если указана папка, то устанавливаются случайные обои из папки
                    random                  Установить случайные обои из всех в списке
                    add <ПУТИ...>           Добавить папку/файл в список и, опционально, настроить параметры (--mode, --recursive-level и т.д.)
                    remove <ПУТИ...>        Удалить папку/файл из списка

                ДЕЙСТВИЯ (Группы):
                    group-list                          Показать список всех групп
                    new-group <ИМЯ> [ПУТИ...]           Создать новую группу
                    get-group <ИМЯ>                     Показать информацию и состав группы
                    set-group <ИМЯ>                     Установить рандомные обои из группы
                    add-to-group <ИМЯ> <ПУТИ...>        Добавить файлы/папки в группу
                    remove-from-group <ИМЯ> <ПУТИ...>   Изъять файлы/папки из группы
                    clear-group <ИМЯ>                   Очистить все элементы внутри группы
                    remove-group <ИМЯ>                  Удалить группу
                
                ДРУГИЕ ДЕЙСТВИЯ:
                    find-non-fitting [ПУТИ...]  Найти все изображения и видео, у которых соотношение сторон отличаеся от монитора
                                                и для которых не задан --mode. Для видео также проверяет попиксельное совпадение
                                                разрешения, так как масштабирование
                                                видео в реальном времени - недешёвая операция.

                ОПЦИИ ВРЕМЕНИ (Для set и set-group):
                    -s, --since <ВРЕМЯ>     Начало действия (по умолчанию: сейчас). Формат: \"14:00\", \"2026-06-11 12:00\"
                    -u, --until <ВРЕМЯ>     Конец действия. Формат: дата/время, now, tomorrow
                    -d, --duration <ДЛИТ>   Длительность применения (например: 30m, 40 minutes, 12h, 3d, 2w, 5 month, 1 year, 20:30).
                                            Месяцы и годы считаются по 30 и 365 дней соответственно.
                    
                    Если опции времени не указаны, обои ставятся только на текущую сессию.

                ОБЩИЕ ОПЦИИ:
                    -m, --mode <РЕЖИМ>  Установить режим отображения обоев. По умолчанию: \"cover center\".
                                    Сторона, к которой будут «прилипать» обои: left, right, top, bottom, center
                                    Отрисовка изображения:
                                    - cover - изображение выходит за границы экрана без искажения пропорций
                                    - contain - изображение полностью помещается в экран без искажения пропорций
                                    - stretch - изображение растягивается под размер экрана с искажением пропорций
                    
                    -r, --recursive-level <УРОВЕНЬ>  Максимальный уровень рекурсивного поиска подпапок. Никак не влияет на файлы. По умолчанию: 1.

                        --display-id     Указать id дисплея (только для find-non-fitting)
                    -h, --help           Показать эту справку
                    -v, --version        Показать версию программы
                ", $cmd)

        } else {

            $crate::indoc::formatdoc! ("
                Desktop wallpaper management utility
                USAGE:
                    {} <ACTION> [OPTIONS] [ARGUMENTS]

                ACTIONS (Wallpapers):
                    list                    Get a list of all wallpaper paths
                    get                     Get the path to the current wallpaper
                    set <PATH>              Set the specified wallpaper. If a folder is specified, random wallpapers from the folder are set
                    random                  Set random wallpapers from all in the list
                    add <PATHS...>          Add a folder/file to the list and, optionally, configure parameters (--mode, --recursive-level, etc.)
                    remove <PATHS...>       Delete a folder/file from the list

                ACTIONS (Groups):
                    group-list                          Show a list of all groups
                    new-group <NAME> [PATHS...]          Create a new group
                    get-group <NAME>                     Show information and composition of the group
                    set-group <NAME>                     Set random wallpapers from the group
                    add-to-group <NAME> <PATHS...>       Add files/folders to the group
                    remove-from-group <NAME> <PATHS...>  Remove files/folders from the group
                    clear-group <NAME>                   Clear all elements within the group
                    remove-group <NAME>                  Delete the group
                
                ANOTHER ACTIONS:
                    find-non-fitting [PATHS...]     Find all images and videos whose aspect ratio differs from the monitor and for which --mode
                                                    is not specified. For videos, it also checks for pixel-by-pixel resolution matching, since
                                                    scaling video in real time is an expensive operation.

                TIME OPTIONS (For set and set-group):
                    -s, --since <TIME>     Start of action (default: now). Format: \"14:00\", \"2026-06-11 12:00\"
                    -u, --until <TIME>     End of action. Format: date/time, now, tomorrow
                    -d, --duration <DURATION>   Duration of application (e.g.: 30m, 40 minutes, 12h, 3d, 2w, 5 month, 1 year, 20:30).
                                                Months and years are counted as 30 and 365 days, respectively.
                    
                    If no time options are specified, the wallpaper is set only for the current session.

                GENERAL OPTIONS:
                    -m, --mode <MODE>  Set the wallpaper display mode. Default: \"cover center\".
                                    The side to which the wallpaper will “stick”: left, right, top, bottom, center
                                    Image rendering:
                                    - cover — the image extends beyond the screen boundaries without distorting proportions
                                    - contain — the image fits entirely within the screen without distorting proportions
                                    - stretch — the image is stretched to fit the screen size with distortion of proportions
                    
                    -r, --recursive-level <LEVEL> Maximum level of recursive subfolder search. Does not affect files in any way. Default: 1.
                    
                        --display-id     Specify the display ID (only for find-non-fitting)
                    -h, --help           Show this help
                    -v, --version        Show the program version
                ", $cmd)

        }
    }
}

pub fn parse_args(args: &Vec<String>) -> Result<Action, ArgParseError> {
    let cmd = &args[0];

    let mut allow_options = true;
    let mut action = ArgParseAction::new();
    let mut iter = args.iter().skip(1);

    while let Some(arg) = iter.next() {
        if allow_options && arg.starts_with("-") {
            let mut opt_name = arg.as_str();
            let mut opt_value = None;

            if opt_name.contains('=') {
                let mut iter = opt_name.splitn(2, '=');
                opt_name = iter.next().unwrap();
                opt_value = Some(iter.next().unwrap());
            }

            match opt_name {
                "-h" | "--help" => {
                    return Ok(Action::Help);
                }

                "-m" | "--mode" => {
                    action.add_setting(arg.clone(), |settings| {
                        settings.mode = Some(parse_display_mode(
                            opt_name,
                            &require_opt_value(opt_name, &mut opt_value, &mut iter)?,
                        )?);
                        Ok(())
                    })?;
                }

                "-r" | "--recursive-level" => {
                    action.add_setting(arg.clone(), |settings| {
                        settings.recursive_level = Some(parse_u16(
                            opt_name,
                            &require_opt_value(opt_name, &mut opt_value, &mut iter)?,
                        )?);
                        Ok(())
                    })?;
                }

                "-s" | "--since" => {
                    action.add_time_period_option(arg.clone(), |period| {
                        period.set_since(time_parser::parse_time_to_minutes(&require_opt_value(
                            opt_name,
                            &mut opt_value,
                            &mut iter,
                        )?)?)
                    })?;
                }

                "-u" | "--until" => {
                    action.add_time_period_option(arg.clone(), |period| {
                        period.set_until(time_parser::parse_time_to_minutes(&require_opt_value(
                            opt_name,
                            &mut opt_value,
                            &mut iter,
                        )?)?)
                    })?;
                }

                "-d" | "--duration" => {
                    action.add_time_period_option(arg.clone(), |period| {
                        period.set_duration(time_parser::parse_duration(&require_opt_value(
                            opt_name,
                            &mut opt_value,
                            &mut iter,
                        )?)?)
                    })?;
                }

                "--display-id" => {
                    let id = parse_u32(
                        opt_name,
                        &require_opt_value(opt_name, &mut opt_value, &mut iter)?,
                    )?;
                    action.add_unique_option(arg.clone(), UniqueOption::DisplayId(id))?;
                }

                "--short" => {
                    action.add_unique_option(arg.clone(), UniqueOption::ShortFlag)?;
                }

                "--" => allow_options = false,

                _ => {
                    return Err(arg_parse_error_localized_with_usage!(
                        "Unknown option: '{opt_name}'",
                        "Неизвестный параметр: '{opt_name}'",
                        cmd
                    ));
                }
            }

            if opt_value.is_some() {}
        } else {
            action.add_arg(arg.clone());
        }
    }

    action.as_action(cmd)
}

fn require_opt_value<'a, I>(opt_name: &str, opt_value: &mut Option<&str>, iter: &mut I) -> Result<String, ArgParseError>
where
    I: Iterator<Item = &'a String>
{
    let cloned = opt_value.clone();

    if opt_value.is_some() {
        *opt_value = None;
    }

    cloned.map(String::from)
        .or_else(|| iter.next().cloned())
        .ok_or_else(|| {
            arg_parse_error_localized!(
                "Missing value for option '{opt_name}'",
                "Требуется значение для опции '{opt_name}'"
            )
        })
}

fn parse_display_mode(opt_name: &str, opt_value: &str) -> Result<DisplayMode, ArgParseError> {
    DisplayMode::from_str(opt_value).map_err(|_| {
        arg_parse_error_localized!(
            "Invalid value for option '{opt_name}': '{opt_value}'",
            "Недопустимое значение для параметра '{opt_name}': '{opt_value}'"
        )
    })
}

fn parse_u16(opt_name: &str, opt_value: &str) -> Result<u16, ArgParseError> {
    opt_value.parse().map_err(
        |_| arg_parse_error_localized!(
            "Invalid value for option '{opt_name}': '{opt_value}'. Expected integer from 0 to 65 535",
            "Недопустимое значение для параметра '{opt_name}': '{opt_value}'. Ожидается целое число от 0 до 65 535"
        )
    )
}

fn parse_u32(opt_name: &str, opt_value: &str) -> Result<u32, ArgParseError> {
    opt_value.parse().map_err(
        |_| arg_parse_error_localized!(
            "Invalid value for option '{opt_name}': '{opt_value}'. Expected integer from 0 to 4 294 967 295",
            "Недопустимое значение для параметра '{opt_name}': '{opt_value}'. Ожидается целое число от 0 до 4 294 967 295"
        )
    )
}
