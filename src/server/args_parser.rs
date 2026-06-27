use crate::server::display_mode::{FitMode, HorizontalAlignment, VerticalAlignment, DisplayMode};
use crate::server::args_parse_error::ArgsParseError;
use crate::server::action::Action;
use crate::args_parse_error_format;

#[macro_export]
macro_rules! GET_HELP_MESSAGE {
    ($cmd:expr) => {
        $crate::indoc::formatdoc! ("
            Утилита управления обоями рабочего стола
            ИСПОЛЬЗОВАНИЕ:
                {} <ДЕЙСТВИЕ> [ОПЦИИ] [АРГУМЕНТЫ]

            ДЕЙСТВИЯ (Обои):
                get                     Получить путь к текущим обоям
                set <ПУТЬ>              Установить указанные обои
                random                  Установить случайные обои
                configure               Настроить параметры обоев (в частности, --mode)

            ДЕЙСТВИЯ (Группы):
                list-groups                         Показать список всех групп
                new-group <ИМЯ>                     Создать новую пустую группу
                get-group <ИМЯ>                     Показать информацию и состав группы
                remove-group <ИМЯ>                  Удалить группу
                set-group <ИМЯ>                     Установить рандомные обои из группы
                add-to-group <ИМЯ> <ПУТИ...>        Добавить файлы/папки в группу
                remove-from-group <ИМЯ> <ПУТИ...>   Изъять файлы/папки из группы
                clear-group <ИМЯ>                   Очистить все элементы внутри группы

            ОПЦИИ ВРЕМЕНИ (Для set и set-group):
                -s, --since <ВРЕМЯ>     Начало действия (по умолчанию: сейчас). Формат: \"14:00\", \"2026-06-11 12:00\"
                -u, --until <ВРЕМЯ>     Конец действия. Формат: дата/время или длительность (\"+2h\", \"+1d\")
                -d, --duration <ДЛИТ>   Длительность применения (например: 30m, 40 minutes, 12h, 3d, 2w, now, tomorrow)
                                        Если опции времени не указаны, обои ставятся только на текущую сессию.

            ОБЩИЕ ОПЦИИ:
                --mode <РЕЖИМ>  Установить режим отображения обоев. По умолчанию: \"cover center\".
                                Сторона, к которой будут «прилипать» обои: left, right, top, bottom, vcenter, hcenter, center
                                Отрисовка изображения:
                                - cover - изображение выходит за границы экрана без искажения пропорций
                                - contain - изображение полностью помещается в экран без искажения пропорций
                                - stretch - изображение растягивается под размер экрана с искажением пропорций
                
                -h, --help           Показать эту справку
                -v, --version        Показать версию программы
            ", $cmd)
    }
}


pub fn parse_args(args: &Vec<String>) -> Result<Action, ArgsParseError> {
    let cmd = &args[0];

    let mut action = Action::None;
    let mut iter = args.iter().skip(1);

    while let Some(arg) = iter.next() {

        if arg.starts_with("-") {
            match arg.as_str() {
                "-h" | "--help" => {
                    return Ok(Action::Help);
                }

                "-m" | "--mode" => {
                    match action {
                        Action::SetWallpaper { ref mut mode, .. } => {
                            match iter.next() {
                                Some(value) => *mode = parse_display_mode(value)?,
                                None => return Err(args_parse_error_format!("Expected value for '{arg}', got end of arguments"))
                            };
                        }
                        
                        Action::Configure { ref mut mode, .. } => {
                            match iter.next() {
                                Some(value) => *mode = parse_display_mode(value)?,
                                None => return Err(args_parse_error_format!("Expected value for '{arg}', got end of arguments"))
                            };
                        }

                        Action::None => return Err(args_parse_error_format!("Cannot use '--mode' before action")),
                        _            => return Err(args_parse_error_format!("Cannot use '--mode' for action {}", action.get_name())),
                    }
                }

                "-s" | "--since" => {} // TODO
                "-u" | "--until" => {} // TODO
                "-d" | "--duration" => {} // TODO

                _ => return Err(args_parse_error_format!("Unrecognized option: '{arg}'. Use '{cmd} --help' to get more information"))
            }
            
        } else {
            
            if action == Action::None {
                action = Action::from_literal(arg.as_str(), cmd.as_str())?;

            } else {
                action.add_arg(arg)?;
            }
        }

    }

    Ok(action)
}


fn parse_display_mode(str: &str) -> Result<DisplayMode, ArgsParseError> {
    let mut mode = DisplayMode::new();

    for token in str.split(" ") {
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

            _ => return Err(args_parse_error_format!("Invalid token in --mode: '{token}'"))
        }
    }

    Ok(mode)
}