use clap::{crate_description, crate_name, crate_version, App, AppSettings, Arg};
use std::os::unix::process::ExitStatusExt;
use std::process;

fn main() {
    let params = App::new(crate_name!())
        .version(crate_version!())
        .long_version(crate_version!())
        .about(crate_description!())
        .long_about(
            "Основное назначение этой программы — запуск команд из скриптов. \
            В выводе скрипта часто бывает нужно видеть, какая именно команда была запущена. \
            Программа runecho сначала показывает запускаемую команду, а затем запускает её.\n\n\
            Команда показывается с приглашением командной строки, что позволяет визуально понять \
            — где команда (ввод), а где результат её выполнения (вывод).",
        )
        .after_help(
            "Программа runecho старается (по возможности) завершаться с тем же кодом выхода, \
            с которым завершилась запускаемая команда.\n\n\
            Если команда была завершена по сигналу (kill -N), то у неё не будет кода выхода. \
            В этом случае runecho завершается с кодом выхода, равным 128+N \
            (это поведение аналогично поведению оболочки bash).",
        )
        // TODO Это костыль, нужно разобраться как настроить правильно
        // Без этого при неправильном запуске выводится кривая подсказка c опцией ПОСЛЕ команды:
        // runecho <COMMAND>... --prompt <STRING>
        .usage("runecho [OPTIONS] <COMMAND>...")
        // Это нужно для того, чтобы отключить парсинг аргументов, идущих после command.
        // Среди них могут быть начинающиеся на тирe и их не нужно парсить, это опции для command.
        .setting(AppSettings::TrailingVarArg)
        .arg(
            Arg::with_name("stderr")
                .help("Показывать запускаемую команду на STDERR вместо STDOUT")
                .short("e")
                .long("stderr")
        )
        .arg(
            Arg::with_name("prompt")
                .help("Приглашение командной строки")
                .value_name("STRING")
                .short("p")
                .long("prompt")
                .default_value(">")
                .allow_hyphen_values(true),
        )
        .arg(
            Arg::with_name("command")
                .help("Запускаемая команда")
                .value_name("COMMAND")
                .required(true)
                .multiple(true),
        )
        .get_matches();

    let prompt = params.value_of("prompt").unwrap();
    let command: Vec<&str> = params.values_of("command").unwrap().collect();

    let cmd = &command[0];
    let args = &command[1..];

    // Показать команду
    if params.is_present("stderr") {
        eprintln!("{} {} {}", prompt, cmd, args.join(" "));
    }
    else {
        println!("{} {} {}", prompt, cmd, args.join(" "));
    }

    // Выполнить команду
    let status = process::Command::new(cmd).args(args).status();

    // Определить код выхода команды
    let code = match status {
        Ok(status) => match status.code() {
            Some(_) => status.code(),
            None => {
                let signal = status.signal().expect("Oops, no signal!");

                eprintln!("{}: terminated by signal {}", cmd, signal);

                Some(128 + signal)
            }
        },
        Err(err) => {
            eprintln!("{}: {}", cmd, err);

            err.raw_os_error()
        }
    };

    // Завершиться с тем же кодом выхода, с которым завершилаcь выполнявшаяся команда
    // TODO На счёт 1 тут сомнительно, по идее такой ситуации не должно возникать
    process::exit(code.unwrap_or(1));
}
