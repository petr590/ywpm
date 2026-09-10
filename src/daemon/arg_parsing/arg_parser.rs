use std::str::FromStr;

use crate::{arg_parse_error_localized, arg_parse_error_localized_with_usage, util};
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
                    list                Получить список всех путей к обоям.
                    get                 Получить путь к текущим обоям.
                    set <ПУТЬ>          Установить указанные обои. Если указана папка, то
                                        устанавливаются случайные обои из папки.
                    random              Установить случайные обои из всех в списке.
                    add <ПУТИ...>       Добавить папку/файл в список и, опционально, настроить
                                        параметры (--mode, --recursive-level и т.д.)
                    remove <ПУТИ...>    Удалить папку/файл из списка (не с диска).
                    restore             Восстановить предыдущие обои (как правило, сервис
                                        systemd автоматически передаёт этот параметр демону
                                        при запуске).

                ДЕЙСТВИЯ (Группы):
                    group-list                  Показать список всех групп.
                    new-group <ИМЯ> [ПУТИ...]   Создать новую группу.
                    get-group <ИМЯ>             Показать информацию и состав группы.
                    set-group <ИМЯ>             Установить рандомные обои из группы.
                    add-to-group <ИМЯ> <ПУТИ...>
                                                Добавить файлы/папки в группу.
                    remove-from-group <ИМЯ> <ПУТИ...>
                                                Удалить файлы/папки из группы (не с диска).
                    clear-group <ИМЯ>           Очистить все элементы внутри группы.
                    remove-group <ИМЯ>          Удалить группу.

                ДРУГИЕ ДЕЙСТВИЯ:
                    find-non-fitting [ПУТИ...]
                                            Найти все изображения и видео, у которых
                                            соотношение сторон отличаеся от монитора и для
                                            которых не задан --mode. Для видео также проверяет
                                            попиксельное совпадение разрешения, так как
                                            масштабирование видео в реальном времени -
                                            недешёвая операция.

                ОПЦИИ ВРЕМЕНИ (Для set и set-group):
                    -s, --since <ВРЕМЯ>     Начало периода (по умолчанию: now).
                                            Формат: \"14:00\", \"2026-06-11 12:00\".
                    -u, --until <ВРЕМЯ>     Конец периода. Формат: дата/время, now, tomorrow
                    -d, --duration <ДЛИТ>   Длительность периода (например: 30m, 40 minutes,
                                            12h, 3d, 2w, 5 month, 1 year, 20:30). Месяцы и годы
                                            считаются по 30 и 365 дней соответственно.

                    Если опции времени не указаны, обои ставятся только на текущую сессию.

                ОБЩИЕ ОПЦИИ:
                    -m, --mode <РЕЖИМ>
                                Установить режим отображения обоев. По умолчанию:
                                \"cover center\". Сторона, к которой будут «прилипать» обои:
                                left, right, top, bottom, center.
                                Отрисовка изображения:
                                - cover   - заполнить экран без искажений пропорций с обрезкой
                                - contain - вписать в экран без искажений пропорций с полями
                                - stretch - растянуть на весь экран с искажением пропорций

                    -r, --recursive-level <УРОВЕНЬ>
                                        Максимальный уровень рекурсивного поиска подпапок.
                                        Никак не влияет на файлы. По умолчанию: 1.
                        --socket        Файл сокета для подключения
                    -v, --verbose       Подробный вывод (только для get и find-non-fitting)
                        --display-id    Указать id дисплея (только для find-non-fitting)
                    -h, --help          Показать эту справку
                    -V, --version       Показать версию программы
                ", $cmd)

        } else {

            $crate::indoc::formatdoc! ("
                Desktop wallpaper management utility
                USAGE:
                    {} <ACTION> [OPTIONS] [ARGUMENTS]

                ACTIONS (Wallpaper):
                    list                Get list of all wallpaper paths.
                    get                 Get path to current wallpaper.
                    set <PATH>          Set specified wallpaper. If folder is specified,
                                        random wallpapers from folder are set.
                    random              Set random wallpapers from all in list.
                    add <PATH...>       Add a folder/file to list and, optionally,
                                        configure parameters (--mode, --recursive-level, etc.)
                    remove <PATH...>    Remove a folder/file from list (not from disk).
                    restore             Restore previous wallpaper (usually, systemd service
                                        automatically passes this parameter to daemon
                                        at startup).

                ACTIONS (Groups):
                    group-list                  Show list of all groups.
                    new-group <NAME> [PATH...]  Create new group.
                    get-group <NAME>            Show information and group's composition.
                    set-group <NAME>            Set random wallpapers from group.
                    add-to-group <NAME> <PATH...>
                                                Add files/folders to group.
                    remove-from-group <NAME> <PATH...>
                                                Remove files/folders from group (not from disk).
                    clear-group <NAME>          Clear all elements within group.
                    remove-group <NAME>         Remove group.

                OTHER ACTIONS:
                    find-non-fitting [PATH...]
                                            Find all images and videos whose aspect ratio
                                            differs from monitor and for which --mode is
                                            not specified. For videos, it also checks for
                                            pixel-by-pixel resolution matching, as real-time
                                            video scaling is expensive operation.
                    
                    help                    Show this help

                TIME OPTIONS (For set and set-group):
                    -s, --since <TIME>      Period start (default: now).
                                            Format: \"14:00\", \"2026-06-11 12:00\".
                    -u, --until <TIME>      Period end. Format: date/time, now, tomorrow
                    -d, --duration <DUR>    Period duration (e.g.: 30m, 40 minutes, 12h, 3d,
                                            2w, 5 month, 1 year, 20:30). Months and
                                            years are counted as 30 and 365 days, respectively.

                If no time options are specified, wallpaper is set only for current session.

                GENERAL OPTIONS:
                    -m, --mode <MODE>
                                Set wallpaper display mode. Default: \"cover center\".
                                Side to which wallpaper will «stick»: left, right, top,
                                bottom, center.
                                Image rendering:
                                - cover   - fill screen, keep aspect ratio, with cropping
                                - contain - fit into screen, keep aspect ratio, with paddings
                                - stretch - stretch to fill screen with distortion

                    -r, --recursive-level <LEVEL>
                                        Maximum level of recursive search for subfolders.
                                        It does not affect files in any way. Default: 1.
                        --socket        Socket file to connect
                    -v, --verbose       Verbose output (only for 'get' and 'find-non-fitting')
                        --display-id    Specify display ID (only for 'find-non-fitting')
                    -h, --help          Show this help
                    -V, --version       Show program version
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
                "-h" | "--help"    => return Ok(Action::Help),
                "-V" | "--version" => return Ok(Action::Version),

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
                        period.set_since(time_parser::parse_time_to_minutes(&require_opt_value(opt_name, &mut opt_value, &mut iter)?)?)
                    })?;
                }

                "-u" | "--until" => {
                    action.add_time_period_option(arg.clone(), |period| {
                        period.set_until(time_parser::parse_time_to_minutes(&require_opt_value(opt_name, &mut opt_value, &mut iter)?)?)
                    })?;
                }

                "-d" | "--duration" => {
                    action.add_time_period_option(arg.clone(), |period| {
                        period.set_duration(time_parser::parse_duration(&require_opt_value(opt_name, &mut opt_value, &mut iter)?)?)
                    })?;
                }

                "--display-id" => {
                    let id = parse_u32(
                        opt_name,
                        &require_opt_value(opt_name, &mut opt_value, &mut iter)?,
                    )?;
                    action.add_unique_option(arg.clone(), UniqueOption::DisplayId(id))?;
                }

                "-v" | "--verbose" => {
                    action.add_unique_option(arg.clone(), UniqueOption::VerboseFlag)?;
                }

                "--socket" => {
                    util::set_socket_path(&require_opt_value(opt_name, &mut opt_value, &mut iter)?);
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
