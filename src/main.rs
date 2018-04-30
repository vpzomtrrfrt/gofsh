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

struct Game {
    draw_pile: Vec<String>,
    my_hand: Vec<String>
}

impl Game {
    fn draw(&mut self) {
        let index = rand::thread_rng().gen_range(0, self.draw_pile.len());
        self.my_hand.push(self.draw_pile.swap_remove(index));
    }

    fn new() -> Self {
        let commands = get_commands();
        Game {
            draw_pile: commands.into_iter()
                .flat_map(|x| vec![x.clone(), x.clone(), x.clone(), x.clone()])
                .collect(),
            my_hand: vec![]
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
                _ => print_help()
            }
        }
    }
}

fn main() {
    Game::new().run();
}
