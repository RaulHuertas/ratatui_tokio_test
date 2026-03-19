use std::{error::Error, io };

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    prelude::Frame,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use reqwest::Client;

use tokio::sync::mpsc;

struct App {
    status: String,
    body: String,
}

enum Message {
    Status(String),
    Body(String),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let (tx, mut rx) = mpsc::channel::<Message>(16);

    tokio::spawn(async move {
        //Http polling rate
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));


        let _ = tx.send(
            Message::Status("I'm Raul. Bye Qt!, I'm doing TUIs now!".to_string())
        ).await;
        let _ = tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await; 
        loop {
            interval.tick().await;

            if tx
                .send(Message::Status("Fetching...".to_string()))
                .await
                .is_err()
            {
                break;
            }

            match fetch_body(&client).await {
                Ok(body) => {
                    if tx.send(Message::Body(body)).await.is_err() {
                        break;
                    }

                    if tx
                        .send(Message::Status("Last fetch: ok".to_string()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                Err(err) => {
                    if tx
                        .send(Message::Status(format!("Last fetch failed: {err}")))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            }
        }
    });

    let mut app = App {
        status: "Starting...".to_string(),
        body: "Waiting for first response...".to_string(),
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let run_result = run_app(&mut terminal, &mut app, &mut rx).await;
    let cleanup_result = restore_terminal(&mut terminal);

    run_result?;
    cleanup_result?;

    Ok(())
}

async fn fetch_body(client: &Client) -> Result<String, reqwest::Error> {
    client
        .get("https://httpbin.org/get")//creates a build request
        .send()//sends the build request
        .await?
        .error_for_status()?
        .text()
        .await
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    rx: &mut mpsc::Receiver<Message>,
) -> io::Result<()> {
    loop {
        while let Ok(message) = rx.try_recv() {
            match message {
                Message::Status(status) => app.status = status,
                Message::Body(body) => app.body = body,
            }
        }
        //Render loop
        terminal.draw(|frame| ui(frame, app))?;

        if event::poll(std::time::Duration::from_millis(9))?
            && let Event::Key(key) = event::read()?{
            match key.code {
                KeyCode::Char('q')=>{
                    return Ok(());//exit application
                }
                _ => {
                    //do nothing
                }
            }
        }
        //UI FRAME RATE
        tokio::time::sleep(tokio::time::Duration::from_millis(7)).await;
    }
}

fn ui(frame: &mut Frame<'_>, app: &App) {
    let chunks = Layout::vertical([Constraint::Length(4), Constraint::Min(1), Constraint::Min(1)]).split(frame.area());

    let status = Paragraph::new(app.status.as_str())
        .block(Block::default().title("Status").borders(Borders::ALL));

    let body = Paragraph::new(app.body.as_str())
        .block(
            Block::default()
                .title("HTTP Response")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: false });

    let test_paragraph = Paragraph::new("Bienvenidos")
        .block(
            Block::default()
                .title("CustomData")
                .borders(Borders::ALL)
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(status, chunks[0]);
    frame.render_widget(body, chunks[1]);
    frame.render_widget(test_paragraph, chunks[2]);
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}
