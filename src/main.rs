extern crate counter;
extern crate rand;

use std::io::Write;
use rand::Rng;

fn get_commands() -> Vec<String> {
    let path = std::env::var("PATH").expect("Missing PATH");
    let output = std::process::Command::new("find")
        .args(path.split(":"))
        .arg("-type")
        .arg("f")
        .arg("-executable")
        .output()
        .expect("Failed to find commands");
    let stdout = String::from_utf8(output.stdout).expect("Failed to parse find output as UTF8");
    return stdout.trim().split("\n").map(|x| {
        std::path::Path::new(x).file_name().expect("Unable to find file name").to_str().unwrap().to_string()
    }).collect();
}

fn print_help() {
    println!("gofsh: [insert help]");
}

fn pager(text: &str) -> Result<(), std::io::Error> {
    let mut child = std::process::Command::new("less")
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    match child.stdin.as_mut() {
        Some(stdin) => {
            stdin.write_all(text.as_bytes())?;
        },
        None => {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "couldn't open stdin"));
        }
    }
    child.wait()?;
    Ok(())
}

fn run_cmd(cmd: &str) -> Result<(), std::io::Error> {
    std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .spawn()?
        .wait()?;
    Ok(())
}

fn draw_into(hand: &mut Vec<String>, draw_pile: &mut Vec<String>) -> String {
    let index = rand::thread_rng().gen_range(0, draw_pile.len());
    let value = draw_pile.swap_remove(index);
    hand.push(value.clone());
    value
}

struct Game {
    commands: Vec<String>,
    draw_pile: Vec<String>,
    my_hand: Vec<String>,
    other_hand: Vec<String>
}

impl Game {
    fn draw(&mut self) -> String {
        draw_into(&mut self.my_hand, &mut self.draw_pile)
    }

    fn draw_other(&mut self) {
        draw_into(&mut self.other_hand, &mut self.draw_pile);
    }

    fn new() -> Self {
        let commands = get_commands();
        Game {
            draw_pile: commands
                .clone()
                .into_iter()
                .flat_map(|x| vec![x.clone(), x.clone(), x.clone(), x.clone()])
                .collect(),
            commands,
            my_hand: vec![],
            other_hand: vec![]
        }
    }

    fn print_hand(&self) {
        let counter = counter::Counter::init(self.my_hand.iter());
        let text = counter.most_common()
            .into_iter()
            .map(|(elem, count)| format!("{:6} {}", count, elem))
            .collect::<Vec<_>>()
            .join("\n");
        pager(&text).unwrap_or_else(|e| eprintln!("{:?}", e));
    }

    fn ask(&mut self, data: &str) {
        let data = data.trim();
        if self.commands.iter().position(|c| c == data).is_none() {
            println!("gofsh: No such card.");
            return;
        }
        let mut received = 0;
        while let Some(pos) = self.other_hand.iter().position(|c| c == data) {
            received += 1;
            self.my_hand.push(self.other_hand.swap_remove(pos));
        }
        if received == 0 {
            println!("gofsh: Go Fish!");
            println!("gofsh: You drew {}", self.draw());
        }
        else {
            println!(
                "gofsh: You received {} {} of {}",
                received,
                if received == 1 {
                    "copy"
                } else {
                    "copies"
                },
                data);
        }
    }

    fn run_cmd(&self, data: &str) {
        let mut spl = data.splitn(2, char::is_whitespace);
        let cmd = spl.next();
        if cmd.is_none() {
            println!("gofsh: Missing command.");
            return;
        }
        let cmd = cmd.unwrap();
        let pos = self.my_hand.iter().position(|c| c == cmd);
        if pos.is_none() {
            println!("gofsh: You don't have that card.");
            return;
        }
        else {
            let pos2 = self.my_hand[(pos.unwrap() + 1)..].iter().position(|c| c == cmd);
            if pos2.is_none() {
                println!("gofsh: You only have one of that card.");
                return;
            }
        }
        run_cmd(data).unwrap_or_else(|e| eprintln!("{:?}", e));
    }

    fn run(&mut self) {
        let start_amount = self.draw_pile.len() / 7;

        for _ in 0..start_amount {
            self.draw();
            self.draw_other();
        }

        let stdin = std::io::stdin();
        loop {
            print!("> ");
            std::io::stdout().flush().unwrap();
            let mut linebuf = String::new();
            stdin.read_line(&mut linebuf).unwrap();

            let mut spl = linebuf.splitn(2, char::is_whitespace);
            let cmd = spl.next().unwrap_or("");
            let data = spl.next().unwrap_or("");

            match cmd {
                "help" => print_help(),
                "hand" => self.print_hand(),
                "run" => self.run_cmd(data),
                "ask" => self.ask(data),
                _ => print_help()
            }
        }
    }
}

fn main() {
    Game::new().run();
}
