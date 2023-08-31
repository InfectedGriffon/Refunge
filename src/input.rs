use std::{fmt::{Debug, Display},io::{self, Write},str::FromStr};

// buffer-less user input
#[allow(unused)]
pub fn take_input(say: impl Display) -> io::Result<String> {
    print!("\x1b[36m{} \x1b[37m\u{203a}\x1b[m ", say);
    io::stdout().flush()?;
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    Ok(buffer.trim_end().to_string())
}

/// repeat reading line until correct format
pub fn take_input_parse<T>(say: impl Display) -> io::Result<T>
    where
        T: FromStr,
        <T as FromStr>::Err: Debug,
{
    let mut buffer = String::new();
    let mut parsed: Result<T, <T as FromStr>::Err>;
    loop {
        print!("\x1b[36m{} \x1b[37m\u{203a}\x1b[m ", say);
        io::stdout().flush()?;
        buffer.clear();
        io::stdin().read_line(&mut buffer)?;
        parsed = buffer.trim().parse();
        match parsed {
            Ok(val) => return Ok(val),
            Err(err) => eprintln!("\x1b[31m{err:?}\x1b[m"),
        }
    }
}
