use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    DefaultTerminal, Frame,
};

use crate::topology::NetworkNode;

/// A flattened row in the tree view for navigation.
#[derive(Clone)]
enum TreeRow {
    Switch { ip: String, sys_descr: Option<String>, depth: usize },
    Port { port_number: u32, device_count: usize, has_child_switch: bool, depth: usize },
    Device { mac: String, ip: Option<String>, is_switch: bool, depth: usize },
}

struct App {
    rows: Vec<TreeRow>,
    list_state: ListState,
}

impl App {
    fn new(root: &NetworkNode) -> Self {
        let mut rows = Vec::new();
        flatten_node(root, 0, &mut rows);
        let mut state = ListState::default();
        if !rows.is_empty() {
            state.select(Some(0));
        }
        App { rows, list_state: state }
    }

    fn next(&mut self) {
        if self.rows.is_empty() { return; }
        let i = match self.list_state.selected() {
            Some(i) => (i + 1) % self.rows.len(),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.rows.is_empty() { return; }
        let i = match self.list_state.selected() {
            Some(i) => if i == 0 { self.rows.len() - 1 } else { i - 1 },
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}

fn flatten_node(node: &NetworkNode, depth: usize, rows: &mut Vec<TreeRow>) {
    rows.push(TreeRow::Switch {
        ip: node.switch_ip.clone(),
        sys_descr: node.sys_descr.clone(),
        depth,
    });

    for port in &node.ports {
        let has_child = port.devices.iter().any(|d| d.is_switch());
        rows.push(TreeRow::Port {
            port_number: port.port_number,
            device_count: port.devices.len(),
            has_child_switch: has_child,
            depth: depth + 1,
        });

        for device in &port.devices {
            rows.push(TreeRow::Device {
                mac: device.mac.clone(),
                ip: device.ip.clone(),
                is_switch: device.is_switch(),
                depth: depth + 2,
            });

            if let Some(ref child) = device.child_switch {
                flatten_node(child, depth + 2, rows);
            }
        }
    }
}

pub fn run(node: NetworkNode) -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run_app(&mut terminal, App::new(&node));
    ratatui::restore();
    result
}

fn run_app(terminal: &mut DefaultTerminal, mut app: App) -> std::io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press { continue; }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Down | KeyCode::Char('j') => app.next(),
                KeyCode::Up | KeyCode::Char('k') => app.previous(),
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(f.area());

    // Left pane: tree view
    let items: Vec<ListItem> = app.rows.iter().map(|row| {
        let indent = match row {
            TreeRow::Switch { depth, .. } |
            TreeRow::Port { depth, .. } |
            TreeRow::Device { depth, .. } => "  ".repeat(*depth),
        };

        let (label, style) = match row {
            TreeRow::Switch { ip, .. } => (
                format!("{}[{}]", indent, ip),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            TreeRow::Port { port_number, device_count, has_child_switch, .. } => {
                let marker = if *has_child_switch { " *" } else { "" };
                (
                    format!("{}Port {} ({} device{}){}", indent, port_number, device_count,
                        if *device_count == 1 { "" } else { "s" }, marker),
                    Style::default().fg(Color::White),
                )
            }
            TreeRow::Device { mac, ip, is_switch, .. } => {
                let ip_str = ip.as_ref().map(|i| format!(" ({})", i)).unwrap_or_default();
                let marker = if *is_switch { " [SW]" } else { "" };
                (
                    format!("{}{}{}{}", indent, mac, ip_str, marker),
                    Style::default().fg(if *is_switch { Color::Yellow } else { Color::DarkGray }),
                )
            }
        };

        ListItem::new(Span::styled(label, style))
    }).collect();

    let tree_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Network Topology "))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("▸ ");

    f.render_stateful_widget(tree_list, chunks[0], &mut app.list_state);

    // Right pane: detail for selected row
    let detail_lines = build_detail(&app.rows, app.list_state.selected());
    let detail = Paragraph::new(detail_lines)
        .block(Block::default().borders(Borders::ALL).title(" Details "));
    f.render_widget(detail, chunks[1]);

    // Help bar
    let help = " [q] Quit  [↑/↓] Navigate ";
    let help_line = Line::from(Span::styled(help, Style::default().fg(Color::DarkGray)));
    let help_area = ratatui::layout::Rect {
        x: f.area().x,
        y: f.area().y + f.area().height.saturating_sub(1),
        width: f.area().width,
        height: 1,
    };
    f.render_widget(Paragraph::new(help_line), help_area);
}

fn build_detail(rows: &[TreeRow], selected: Option<usize>) -> Vec<Line<'static>> {
    let Some(idx) = selected else {
        return vec![Line::from("  Select an item")];
    };

    match &rows[idx] {
        TreeRow::Switch { ip, sys_descr, .. } => {
            let mut lines = vec![
                Line::from(Span::styled(
                    format!("Switch: {}", ip),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
            ];
            if let Some(descr) = sys_descr {
                lines.push(Line::from(vec![
                    Span::styled("  sysDescr: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(descr.clone(), Style::default().fg(Color::White)),
                ]));
            }
            lines
        }
        TreeRow::Port { port_number, device_count, .. } => {
            vec![
                Line::from(Span::styled(
                    format!("Port {}", port_number),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(format!("  {} device{}", device_count, if *device_count == 1 { "" } else { "s" })),
            ]
        }
        TreeRow::Device { mac, ip, is_switch, .. } => {
            let mut lines = vec![
                Line::from(vec![
                    Span::styled("  MAC: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(mac.clone(), Style::default().fg(Color::White)),
                ]),
            ];
            if let Some(ip) = ip {
                lines.push(Line::from(vec![
                    Span::styled("   IP: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(ip.clone(), Style::default().fg(Color::Green)),
                ]));
            }
            if *is_switch {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "  This device is an SNMP-managed switch",
                    Style::default().fg(Color::Yellow),
                )));
            }
            lines
        }
    }
}
