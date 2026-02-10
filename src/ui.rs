use crate::blm::melhor_melhora;
use crate::blnm::busca_local_iterada;
use crate::utils::{salvar_csv, Result};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
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
    pub selected_algorithm: usize,
    pub selected_m: usize,
    pub selected_r: usize,
    pub selected_perturbacao: usize,
    pub selected_max_iter: usize,
    pub m_values: Vec<usize>,
    pub r_values: Vec<f64>,
    pub perturbacao_values: Vec<f64>,
    pub max_iter_values: Vec<u32>,
    pub results: Vec<Result>,
    pub current_exec: usize,
    pub output_filename: String,
    pub perturbacao_state: ListState,
    pub max_iter_state: ListState,
    pub scroll_position: u16,
}

impl App {
    pub fn new() -> Self {
        let mut perturbacao_state = ListState::default();
        perturbacao_state.select(Some(2));
        let mut max_iter_state = ListState::default();
        max_iter_state.select(Some(3));

        App {
            current_screen: Screen::Menu,
            selected_algorithm: 0,
            selected_m: 0,
            selected_r: 0,
            selected_perturbacao: 2,
            selected_max_iter: 3,
            m_values: vec![10, 20, 50],
            r_values: vec![1.5, 2.0],
            perturbacao_values: vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9],
            max_iter_values: vec![0, 100, 500, 1000, 2000, 5000],
            results: Vec::new(),
            current_exec: 0,
            output_filename: "resultados.csv".to_string(),
            perturbacao_state,
            max_iter_state,
            scroll_position: 0,
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
            Constraint::Length(5),
            Constraint::Length(if app.selected_algorithm == 1 { 7 } else { 0 }),
            Constraint::Length(if app.selected_algorithm == 1 { 7 } else { 0 }),
            Constraint::Min(0),
        ])
        .split(area);

    let algorithm_names = [
        "Busca Local Monotônica - Melhor Melhora",
        "Busca Local Iterada",
    ];
    let title = Paragraph::new(algorithm_names[app.selected_algorithm])
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, menu_chunks[0]);

    let algo_items: Vec<ListItem> = algorithm_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let style = if i == app.selected_algorithm {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(name.to_string()).style(style)
        })
        .collect();

    let algo_list = List::new(algo_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Algoritmo (Tab)"),
    );
    f.render_widget(algo_list, menu_chunks[1]);

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
            let prefix = if i == app.selected_m { "► " } else { "  " };
            ListItem::new(format!("{prefix}Máquinas: {m}")).style(style)
        })
        .collect();

    let m_title = format!(
        "Número de Máquinas (↑/↓) [{}/{}]",
        app.selected_m + 1,
        app.m_values.len()
    );
    let m_list = List::new(m_items).block(Block::default().borders(Borders::ALL).title(m_title));
    f.render_widget(m_list, menu_chunks[2]);

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
            let prefix = if i == app.selected_r { "► " } else { "  " };
            ListItem::new(format!("{prefix}Replicação: {r}")).style(style)
        })
        .collect();

    let r_title = format!(
        "Fator de Replicação (←/→) [{}/{}]",
        app.selected_r + 1,
        app.r_values.len()
    );
    let r_list = List::new(r_items).block(Block::default().borders(Borders::ALL).title(r_title));
    f.render_widget(r_list, menu_chunks[3]);

    // Mostrar perturbação apenas se ILS estiver selecionado
    if app.selected_algorithm == 1 {
        let pert_items: Vec<ListItem> = app
            .perturbacao_values
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let style = if i == app.selected_perturbacao {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                let prefix = if i == app.selected_perturbacao {
                    "► "
                } else {
                    "  "
                };
                let scroll_hint = if i == 0 && app.perturbacao_values.len() > 1 {
                    " ▼"
                } else if i == app.perturbacao_values.len() - 1 && app.perturbacao_values.len() > 1
                {
                    " ▲"
                } else if i == app.selected_perturbacao && app.perturbacao_values.len() > 1 {
                    if i > 0 && i < app.perturbacao_values.len() - 1 {
                        " ▲▼"
                    } else {
                        ""
                    }
                } else {
                    ""
                };
                ListItem::new(format!("{prefix}Perturbação: {p}{scroll_hint}")).style(style)
            })
            .collect();

        let pert_title = format!(
            "Intensidade de Perturbação (W/S) [{}/{}] {}",
            app.selected_perturbacao + 1,
            app.perturbacao_values.len(),
            if app.perturbacao_values.len() > 1 {
                "↕"
            } else {
                ""
            }
        );
        let pert_list = List::new(pert_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(pert_title)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_stateful_widget(
            pert_list,
            menu_chunks[4],
            &mut app.perturbacao_state.clone(),
        );

        let max_iter_items: Vec<ListItem> = app
            .max_iter_values
            .iter()
            .enumerate()
            .map(|(i, val)| {
                let style = if i == app.selected_max_iter {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                let prefix = if i == app.selected_max_iter {
                    "► "
                } else {
                    "  "
                };
                let scroll_hint = if i == 0 && app.max_iter_values.len() > 1 {
                    " ▼"
                } else if i == app.max_iter_values.len() - 1 && app.max_iter_values.len() > 1 {
                    " ▲"
                } else if i == app.selected_max_iter && app.max_iter_values.len() > 1 {
                    if i > 0 && i < app.max_iter_values.len() - 1 {
                        " ▲▼"
                    } else {
                        ""
                    }
                } else {
                    ""
                };
                ListItem::new(format!("{prefix}Max Iter s/ Melhora: {val}{scroll_hint}"))
                    .style(style)
            })
            .collect();

        let max_iter_title = format!(
            "Critério de Parada (A/D) [{}/{}] {}",
            app.selected_max_iter + 1,
            app.max_iter_values.len(),
            if app.max_iter_values.len() > 1 {
                "↕"
            } else {
                ""
            }
        );
        let max_iter_list = List::new(max_iter_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(max_iter_title)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_stateful_widget(
            max_iter_list,
            menu_chunks[5],
            &mut app.max_iter_state.clone(),
        );
    }

    let help_idx = if app.selected_algorithm == 1 { 6 } else { 4 };
    let help = Paragraph::new(vec![
        Line::from("Pressione ENTER para executar | Q para sair"),
        Line::from(Span::styled(
            format!("Arquivo de saída: {}", app.output_filename),
            Style::default().fg(Color::Gray),
        )),
    ])
    .style(Style::default().fg(Color::Green))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, menu_chunks[help_idx]);
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
            let mut lines = vec![
                Line::from(format!("=== Resultado {} ===", i + 1)),
                Line::from(format!("Algoritmo: {}", r.algoritmo)),
                Line::from(format!(
                    "Tarefas: {} | Máquinas: {} | Replicação: {}",
                    r.n_tarefas, r.n_maquinas, r.replicacao
                )),
            ];
            if r.perturbacao > 0.0 {
                lines.push(Line::from(format!("Perturbação: {:.1}", r.perturbacao)));
            }
            lines.extend(vec![
                Line::from(format!(
                    "Tempo: {:.2}ms | Iterações: {}",
                    r.tempo_exec, r.iteracoes
                )),
                Line::from(format!(
                    "Makespan: {} → {}",
                    r.makespan_inicial, r.makespan_final
                )),
                Line::from(""),
            ]);
            lines
        })
        .chain(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("Resultados salvos em: {}", app.output_filename),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Pressione ↑/↓ para rolar | ENTER para voltar ao menu",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
        ])
        .collect();

    let total_lines = results_text.len() as u16;
    let visible_lines = area.height.saturating_sub(2); // subtract borders
    let can_scroll_up = app.scroll_position > 0;
    let can_scroll_down = app.scroll_position + visible_lines < total_lines;

    let scroll_indicator = if can_scroll_up && can_scroll_down {
        " ↕ SCROLL"
    } else if can_scroll_up {
        " ↑ SCROLL UP"
    } else if can_scroll_down {
        " ↓ SCROLL DOWN"
    } else {
        ""
    };

    let title = format!("Resultados{scroll_indicator}");
    let paragraph = Paragraph::new(results_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(if scroll_indicator.is_empty() {
                    Style::default()
                } else {
                    Style::default().fg(Color::Cyan)
                }),
        )
        .scroll((app.scroll_position, 0));
    f.render_widget(paragraph, area);
}

fn handle_input(app: &mut App, key_code: KeyCode) -> io::Result<()> {
    match app.current_screen {
        Screen::Menu => match key_code {
            KeyCode::Char('q') => std::process::exit(0),
            KeyCode::Tab => {
                app.selected_algorithm = if app.selected_algorithm == 0 { 1 } else { 0 };
            }
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
            KeyCode::Char('w') | KeyCode::Char('W') => {
                if app.selected_algorithm == 1 && app.selected_perturbacao > 0 {
                    app.selected_perturbacao -= 1;
                    app.perturbacao_state.select(Some(app.selected_perturbacao));
                }
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                if app.selected_algorithm == 1
                    && app.selected_perturbacao < app.perturbacao_values.len() - 1
                {
                    app.selected_perturbacao += 1;
                    app.perturbacao_state.select(Some(app.selected_perturbacao));
                }
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                if app.selected_algorithm == 1 && app.selected_max_iter > 0 {
                    app.selected_max_iter -= 1;
                    app.max_iter_state.select(Some(app.selected_max_iter));
                }
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if app.selected_algorithm == 1
                    && app.selected_max_iter < app.max_iter_values.len() - 1
                {
                    app.selected_max_iter += 1;
                    app.max_iter_state.select(Some(app.selected_max_iter));
                }
            }
            KeyCode::Enter => {
                app.current_screen = Screen::Running;
                app.results.clear();
                app.current_exec = 0;
                app.scroll_position = 0;
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
                app.scroll_position = 0;
            }
            KeyCode::Up => {
                if app.scroll_position > 0 {
                    app.scroll_position -= 1;
                }
            }
            KeyCode::Down => {
                app.scroll_position += 1;
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

    let result = if app.selected_algorithm == 0 {
        melhor_melhora(m, n, r)
    } else {
        let perturbacao = app.perturbacao_values[app.selected_perturbacao];
        let max_iter = app.max_iter_values[app.selected_max_iter];
        busca_local_iterada(m, n, r, perturbacao, max_iter)
    };

    // Save to CSV
    if let Err(e) = salvar_csv(&result, &app.output_filename) {
        eprintln!("Erro ao salvar arquivo: {e}");
    }

    app.results.push(result);
    app.current_exec += 1;

    if app.current_exec >= 10 {
        app.current_screen = Screen::Results;
    }
}
