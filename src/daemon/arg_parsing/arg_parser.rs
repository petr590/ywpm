use crate::{arg_parse_error_localized, arg_parse_error_localized_with_usage};
use crate::daemon::action::Action;
use crate::daemon::arg_parsing::action::ArgParseAction;
use crate::daemon::arg_parsing::error::ArgParseError;
use crate::daemon::arg_parsing::time_parser;
use crate::daemon::display_mode::{FitMode, HorizontalAlignment, VerticalAlignment, DisplayMode};

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

                ОПЦИИ ВРЕМЕНИ (Для set и set-group):
                    -s, --since <ВРЕМЯ>     Начало действия (по умолчанию: сейчас). Формат: \"14:00\", \"2026-06-11 12:00\"
                    -u, --until <ВРЕМЯ>     Конец действия. Формат: дата/время или длительность (\"+2h\", \"+1d\")
                    -d, --duration <ДЛИТ>   Длительность применения (например: 30m, 40 minutes, 12h, 3d, 2w, 5 month, 1 year, 20:30, tomorrow).
                                            Месяцы и годы считаются по 30 и 365 дней соответственно.
                                            Если опции времени не указаны, обои ставятся только на текущую сессию.

                ОБЩИЕ ОПЦИИ:
                    -m, --mode <РЕЖИМ>  Установить режим отображения обоев. По умолчанию: \"cover center\".
                                    Сторона, к которой будут «прилипать» обои: left, right, top, bottom, vcenter, hcenter, center
                                    Отрисовка изображения:
                                    - cover - изображение выходит за границы экрана без искажения пропорций
                                    - contain - изображение полностью помещается в экран без искажения пропорций
                                    - stretch - изображение растягивается под размер экрана с искажением пропорций
                    
                    -r, --recursive-level <УРОВЕНЬ>  Максимальный уровень рекурсивного поиска подпапок. Никак не влияет на файлы. По умрлчанию: 1.
                    
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

                TIME OPTIONS (For set and set-group):
                    -s, --since <TIME>     Start of action (default: now). Format: \"14:00\", \"2026-06-11 12:00\"
                    -u, --until <TIME>     End of action. Format: date/time or duration (\"+2h\", \"+1d\")
                    -d, --duration <DURATION>   Duration of application (e.g.: 30m, 40 minutes, 12h, 3d, 2w, 5 month, 1 year, 20:30, tomorrow).
                                                Months and years are counted as 30 and 365 days, respectively.
                                                If no time options are specified, the wallpaper is set only for the current session.

                GENERAL OPTIONS:
                    -m, --mode <MODE>  Set the wallpaper display mode. Default: \"cover center\".
                                    The side to which the wallpaper will “stick”: left, right, top, bottom, vcenter, hcenter, center
                                    Image rendering:
                                    - cover — the image extends beyond the screen boundaries without distorting proportions
                                    - contain — the image fits entirely within the screen without distorting proportions
                                    - stretch — the image is stretched to fit the screen size with distortion of proportions
                    
                    -r, --recursive-level <LEVEL> Maximum level of recursive subfolder search. Does not affect files in any way. Default: 1.
                    
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
            let opt_name = arg.as_str();

            if opt_name.contains('=') {
                // TODO
            }

            match opt_name {
                "-h" | "--help" => {
                    return Ok(Action::Help);
                }

                "-m" | "--mode" => {
                    action.add_setting(arg.clone(), |settings| {
                        settings.mode = Some(parse_display_mode(opt_name, require_opt_value(opt_name, iter.next())?)?);
                        Ok(())
                    })?;
                }

                "-r" | "--recursive-level" => {
                    action.add_setting(arg.clone(), |settings| {
                        settings.recursive_level = Some(parse_u16(opt_name, require_opt_value(opt_name, iter.next())?)?);
                        Ok(())
                    })?;
                }

                "-s" | "--since" => {
                    action.add_time_period_option(
                        arg.clone(),
                        |period| period.set_since(time_parser::parse_time_to_minutes(require_opt_value(opt_name, iter.next())?)?)
                    )?;
                },

                "-u" | "--until" => {
                    action.add_time_period_option(
                        arg.clone(),
                        |period| period.set_until(time_parser::parse_time_to_minutes(require_opt_value(opt_name, iter.next())?)?)
                    )?;
                },

                "-d" | "--duration" => {
                    action.add_time_period_option(
                        arg.clone(),
                        |period| period.set_duration(time_parser::parse_duration(require_opt_value(opt_name, iter.next())?)?)
                    )?;
                },

                "--" => allow_options = false,

                _ => return Err(arg_parse_error_localized_with_usage!(
                    "Unknown option: '{opt_name}'",
                    "Неизвестный параметр: '{opt_name}'",
                    cmd
                ))
            }
            
        } else {
            action.add_arg(arg.clone());
        }

    }

    action.as_action(cmd)
}


fn require_opt_value<'a>(opt_name: &str, opt_value: Option<&'a String>) -> Result<&'a String, ArgParseError> {
    opt_value.ok_or_else(|| arg_parse_error_localized!(
        "Missing value for option '{opt_name}'",
        "Требуется значение для опции '{opt_name}'"
    ))
}


fn parse_u16(opt_name: &str, opt_value: &str) -> Result<u16, ArgParseError> {
    opt_value.parse().map_err(
        |_| arg_parse_error_localized!(
            "Invalid value for option '{opt_name}': '{opt_value}'. Expected integer from 0 to 65535",
            "Недопустимое значение для параметра '{opt_name}': '{opt_value}'. Ожидается целое число от 0 до 65535"
        )
    )
}


fn parse_display_mode(opt_name: &str, opt_value: &str) -> Result<DisplayMode, ArgParseError> {
    let mut mode = DisplayMode::new();

    for token in opt_value.split(" ") {
        match token {
            "cover"   => mode.fit_mode = FitMode::Cover,
            "contain" => mode.fit_mode = FitMode::Contain,
            "stretch" => mode.fit_mode = FitMode::Stretch,

            "left"    => mode.h_align = HorizontalAlignment::Left,
            "hcenter" => mode.h_align = HorizontalAlignment::Center,
            "right"   => mode.h_align = HorizontalAlignment::Right,

            "top"     => mode.v_align = VerticalAlignment::Top,
            "vcenter" => mode.v_align = VerticalAlignment::Center,
            "bottom"  => mode.v_align = VerticalAlignment::Bottom,

            "center" => {
                mode.h_align = HorizontalAlignment::Center;
                mode.v_align = VerticalAlignment::Center;
            }

            _ => return Err(arg_parse_error_localized!(
                "Invalid value for option '{opt_name}': '{token}'",
                "Недопустимое значение для параметра '{opt_name}': '{opt_value}'"
            ))
        }
    }

    Ok(mode)
}