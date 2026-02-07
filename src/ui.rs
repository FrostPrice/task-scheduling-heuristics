use crate::blm::{melhor_melhora, BLMResult};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;
use std::time::Duration;

pub enum Screen {
    Menu,
    Running,
    Results,
}

pub struct App {
    pub current_screen: Screen,
    pub selected_m: usize,
    pub selected_r: usize,
    pub m_values: Vec<usize>,
    pub r_values: Vec<f64>,
    pub results: Vec<BLMResult>,
    pub current_exec: usize,
    pub output_filename: String,
}

impl App {
    pub fn new() -> Self {
        App {
            current_screen: Screen::Menu,
            selected_m: 0,
            selected_r: 0,
            m_values: vec![10, 20, 50],
            r_values: vec![1.5, 2.0],
            results: Vec::new(),
            current_exec: 0,
            output_filename: "resultados_blm.csv".to_string(),
        }
    }
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0)])
                .split(f.size());

            match app.current_screen {
                Screen::Menu => {
                    render_menu(f, &app, chunks[0]);
                }
                Screen::Running => {
                    render_running(f, &app, chunks[0]);
                }
                Screen::Results => {
                    render_results(f, &app, chunks[0]);
                }
            }
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                handle_input(&mut app, key.code)?;
            }
        }

        if matches!(app.current_screen, Screen::Running) && app.current_exec < 10 {
            execute_blm(&mut app);
        }
    }
}

fn render_menu(f: &mut ratatui::Frame, app: &App, area: ratatui::layout::Rect) {
    let menu_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(area);

    let title = Paragraph::new("Busca Local Monotônica - Melhor Melhora")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, menu_chunks[0]);

    let m_items: Vec<ListItem> = app
        .m_values
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let style = if i == app.selected_m {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(format!("Máquinas: {m}")).style(style)
        })
        .collect();

    let m_list = List::new(m_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Número de Máquinas (↑/↓)"),
    );
    f.render_widget(m_list, menu_chunks[1]);

    let r_items: Vec<ListItem> = app
        .r_values
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let style = if i == app.selected_r {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(format!("Replicação: {r}")).style(style)
        })
        .collect();

    let r_list = List::new(r_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Fator de Replicação (←/→)"),
    );
    f.render_widget(r_list, menu_chunks[2]);

    let help = Paragraph::new(vec![
        Line::from("Pressione ENTER para executar | Q para sair"),
        Line::from(Span::styled(
            format!("Arquivo de saída: {}", app.output_filename),
            Style::default().fg(Color::Gray),
        )),
    ])
    .style(Style::default().fg(Color::Green))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, menu_chunks[3]);
}

fn render_running(f: &mut ratatui::Frame, app: &App, area: ratatui::layout::Rect) {
    let text = vec![
        Line::from(Span::styled(
            "Executando algoritmo...",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(format!("Execução: {}/10", app.current_exec)),
        Line::from(""),
        Line::from(format!("Salvando em: {}", app.output_filename)),
        Line::from(""),
        Line::from("Pressione Q para cancelar"),
    ];
    let paragraph =
        Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Progresso"));
    f.render_widget(paragraph, area);
}

fn render_results(f: &mut ratatui::Frame, app: &App, area: ratatui::layout::Rect) {
    let results_text: Vec<Line> = app
        .results
        .iter()
        .enumerate()
        .flat_map(|(i, r)| {
            vec![
                Line::from(format!("=== Resultado {} ===", i + 1)),
                Line::from(format!(
                    "Tarefas: {} | Máquinas: {} | Replicação: {}",
                    r.n_tarefas, r.n_maquinas, r.replicacao
                )),
                Line::from(format!(
                    "Tempo: {:.2}ms | Iterações: {}",
                    r.tempo_exec, r.iteracoes
                )),
                Line::from(format!(
                    "Makespan: {} → {}",
                    r.makespan_inicial, r.makespan_final
                )),
                Line::from(""),
            ]
        })
        .chain(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("✓ Resultados salvos em: {}", app.output_filename),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Pressione ENTER para voltar ao menu"),
        ])
        .collect();

    let paragraph = Paragraph::new(results_text)
        .block(Block::default().borders(Borders::ALL).title("Resultados"));
    f.render_widget(paragraph, area);
}

fn handle_input(app: &mut App, key_code: KeyCode) -> io::Result<()> {
    match app.current_screen {
        Screen::Menu => match key_code {
            KeyCode::Char('q') => std::process::exit(0),
            KeyCode::Up => {
                if app.selected_m > 0 {
                    app.selected_m -= 1;
                }
            }
            KeyCode::Down => {
                if app.selected_m < app.m_values.len() - 1 {
                    app.selected_m += 1;
                }
            }
            KeyCode::Left => {
                if app.selected_r > 0 {
                    app.selected_r -= 1;
                }
            }
            KeyCode::Right => {
                if app.selected_r < app.r_values.len() - 1 {
                    app.selected_r += 1;
                }
            }
            KeyCode::Enter => {
                app.current_screen = Screen::Running;
                app.results.clear();
                app.current_exec = 0;
            }
            _ => {}
        },
        Screen::Running => {
            if let KeyCode::Char('q') = key_code {
                app.current_screen = Screen::Menu;
            }
        }
        Screen::Results => match key_code {
            KeyCode::Char('q') | KeyCode::Enter => {
                app.current_screen = Screen::Menu;
            }
            _ => {}
        },
    }
    Ok(())
}

fn execute_blm(app: &mut App) {
    let m = app.m_values[app.selected_m];
    let r = app.r_values[app.selected_r];
    let n = (m as f64).powf(r) as usize;

    let result = melhor_melhora(m, n, r);

    // Save to CSV
    if let Err(e) = result.save_to_csv(&app.output_filename) {
        eprintln!("Erro ao salvar arquivo: {e}");
    }

    app.results.push(result);
    app.current_exec += 1;

    if app.current_exec >= 10 {
        app.current_screen = Screen::Results;
    }
}
