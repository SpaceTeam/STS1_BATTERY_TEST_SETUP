use crate::ui::AppEvent;

pub fn try_parse(input: &String) -> Option<AppEvent> {
    let mut input = input.trim().split_whitespace();
    let command = input.next().unwrap();
    let args = input.collect::<Vec<&str>>();
    match command {
        "ping" => {
            if args.len() == 1 {
                let number = match args[0].parse::<u16>() {
                    Ok(number) => number,
                    Err(_) => return None,
                };
                Some(AppEvent::SendPing(number))
            } else {
                None
            }
        }
        "adc" => {
            if args.len() == 1 {
                let number = match args[0].parse::<u8>() {
                    Ok(number) => number,
                    Err(_) => return None,
                };
                Some(AppEvent::SampleAdc(number))
            } else {
                None
            }
        }
        "quit" => Some(AppEvent::Quit),
        _ => None,
    }
}
