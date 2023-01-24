use tokio::fs::{read_to_string, write as write_file};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader, Stdin, Stdout};

#[tokio::main]
async fn main() -> io::Result<()> {
    run().await?;
    Ok(())
}

async fn run() -> io::Result<()> {
    let path = "src/notes.txt".to_string();
    let mut terminal = Terminal::new();
    let mut storage = NoteStorage::new(path).await?;
    
    loop {
        terminal.start().await?;
        match terminal.show_user_options().await? {
            UserOptions::Insert => {
                let note = terminal.ask_new_note().await?;
                storage.insert(note.note).await?;
                terminal.clear();
            }
            UserOptions::Read => {
                terminal.clear();
                terminal.show_notes(&mut storage).await?;
            }
            UserOptions::Exit => return Ok(()),
            UserOptions::Other => {
                terminal.clear();
                terminal.user_option_invalid().await?;
            }
        }
    }
}

pub(crate) struct Notes {
    note: String,
}

pub(crate) struct Terminal {
    input: BufReader<Stdin>,
    output: Stdout,
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            input: BufReader::new(tokio::io::stdin()),
            output: tokio::io::stdout(),
        }
    }

    async fn start(&mut self) -> io::Result<()> {
        self.print("\n********* SEU DIÁRIO *********\n").await?;
        Ok(())
    }

    async fn user_chosen_option(&mut self, option: String) -> io::Result<UserOptions> {
        match option.trim() {
            "1" => Ok(UserOptions::Insert),
            "2" => Ok(UserOptions::Read),
            "3" => Ok(UserOptions::Exit),
            _ => Ok(UserOptions::Other),
        }
    }

    async fn show_user_options(&mut self) -> io::Result<UserOptions> {
        self.print("\nEscolha uma opção:\n").await?;
        self.print(
            "
1 - Adicionar uma nota ao seu Diário
2 - Ver o seu Diário
3 - Sair
        ",
        )
        .await?;
        let user_response = self.get_user_in().await?;
        self.user_chosen_option(user_response).await
    }

    async fn ask_new_note(&mut self) -> io::Result<Notes> {
        self.print("\nDigite uma nota que deseja adicionar ao seu Diario:\n")
            .await?;
        let user_note = self.get_user_in().await?;
        Ok(Notes { note: user_note })
    }

    async fn show_notes(&mut self, notes: &mut NoteStorage) -> io::Result<()> {
        let all_notes = notes.read_notes().await?;
        self.print("\nO seu diario contém:\n\n").await?;
        self.output.write(&all_notes.as_bytes()).await?;

        Ok(())
    }

    async fn user_option_invalid(&mut self) -> io::Result<()> {
        self.print("\nO comando digitado é inválido!\n").await?;
        Ok(())
    }

    async fn print(&mut self, message: &str) -> io::Result<()> {
        self.output.write(message.as_bytes()).await?;
        Ok(())
    }

    async fn get_user_in(&mut self) -> io::Result<String> {
        let mut buf = String::new();
        self.input.read_line(&mut buf).await?;
        Ok(buf)
    }

    fn clear(&self) {
        print!("{esc}c", esc = 27 as char);
    }
}

struct NoteStorage {
    notes: String,
    path: String,
}

impl NoteStorage {
    async fn new(path_file: String) -> io::Result<Self> {
        Ok(Self {
            notes: read_to_string(&path_file).await?,
            path: path_file,
        })
    }

    async fn insert(&mut self, note: String) -> io::Result<()> {
        let notes_complete = format!("{}{}", self.notes, note);
        write_file(&self.path, &notes_complete.as_bytes()).await?;
        self.notes = notes_complete;
        Ok(())
    }

    async fn read_notes(&mut self) -> io::Result<String> {
        let notes = read_to_string(&self.path).await?;
        Ok(notes)
    }
}

enum UserOptions {
    Insert,
    Read,
    Exit,
    Other,
}
