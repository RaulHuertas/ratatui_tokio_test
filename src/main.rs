use std::{error::Error, io};

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout, Rect},
    prelude::Frame,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use reqwest::Client;

use tokio::sync::mpsc;

struct App {
    status: String,
    body: String,
    button_rect: Rect,
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

        let _ = tx
            .send(Message::Status(
                "I'm Raul. Bye Qt!, I'm doing TUIs now!".to_string(),
            ))
            .await;
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
        button_rect: Rect::default(),
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
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
        .get("https://httpbin.org/get") //creates a build request
        .send() //sends the build request
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
            && let Some(should_quit) = handle_event(event::read()?, app)
            && should_quit
        {
            return Ok(());
        }
        //UI FRAME RATE
        tokio::time::sleep(tokio::time::Duration::from_millis(7)).await;
    }
}

fn handle_event(event: Event, app: &mut App) -> Option<bool> {
    match event {
        Event::Key(key) => match key.code {
            KeyCode::Char('q') => Some(true),
            _ => Some(false),
        },
        Event::Mouse(mouse)
            if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left))
                && rect_contains(app.button_rect, mouse.column, mouse.row) =>
        {
            app.status = "Hi rodrigo".to_string();
            Some(false)
        }
        _ => None,
    }
}

fn rect_contains(rect: Rect, column: u16, row: u16) -> bool {
    column >= rect.x
        && column < rect.x.saturating_add(rect.width)
        && row >= rect.y
        && row < rect.y.saturating_add(rect.height)
}

fn ui(frame: &mut Frame<'_>, app: &mut App) {
    let chunks = Layout::vertical([
        Constraint::Length(4),
        Constraint::Min(1),
        Constraint::Length(5),
        Constraint::Length(3),
    ])
    .split(frame.area());

    let button_row = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(22),
        Constraint::Fill(1),
    ])
    .split(chunks[3]);

    app.button_rect = button_row[1];

    let status = Paragraph::new(app.status.as_str())
        .style(Style::default().fg(Color::Green))
        .block(
            Block::default()
                .title("Status")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(180, 120, 220))),
        );

    let body = Paragraph::new(app.body.as_str())
        .block(
            Block::default()
                .title("HTTP Response")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: false });

    let test_paragraph = Paragraph::new("Bienvenidos")
        .block(Block::default().title("CustomData").borders(Borders::ALL))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });

    let button = Paragraph::new("Click Me")
        .block(
            Block::default().
            title("Action").
            borders(Borders::ALL).
            border_style(Style::default().fg(Color::Rgb(100, 12, 250)))
        )
        .alignment(Alignment::Center);

    frame.render_widget(status, chunks[0]);
    frame.render_widget(body, chunks[1]);
    frame.render_widget(test_paragraph, chunks[2]);
    frame.render_widget(button, app.button_rect);
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()
}
