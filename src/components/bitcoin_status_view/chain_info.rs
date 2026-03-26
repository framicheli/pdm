use crate::app::App;
use anyhow::Result;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
};
use serde::Deserialize;
use std::process::Command;

fn format_size(bytes: u64) -> String {
    const GB: u64 = 1_000_000_000;
    const MB: u64 = 1_000_000;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

#[derive(Debug, Clone)]
pub struct ChainInfo {
    #[allow(dead_code)]
    pub error: Option<String>,
    pub data: Option<BlockchainInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlockchainInfo {
    pub chain: String,
    pub blocks: u64,
    pub headers: u64,
    pub bestblockhash: String,
    pub difficulty: f64,
    pub time: u64,
    pub mediantime: u64,
    pub verificationprogress: f64,
    pub initialblockdownload: bool,
    pub chainwork: String,
    pub size_on_disk: u64,
    pub pruned: bool,
    pub warnings: String,
}

impl ChainInfo {
    pub fn new() -> Self {
        ChainInfo {
            error: None,
            data: None,
        }
    }

    pub fn fetch(&mut self) {
        match Self::run_bitcoin_cli() {
            Ok(info) => {
                self.data = Some(info);
                self.error = None;
            }
            Err(e) => {
                self.data = None;
                self.error = Some(e.to_string());
            }
        }
    }

    fn run_bitcoin_cli() -> Result<BlockchainInfo> {
        let output = Command::new("bitcoin-cli")
            .arg("getblockchaininfo")
            .output()
            .map_err(|e| anyhow::anyhow!("bitcoin-cli not found: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("{}", stderr.trim());
        }

        let info: BlockchainInfo = serde_json::from_slice(&output.stdout)?;
        Ok(info)
    }

    pub fn render(f: &mut Frame, app: &App, area: Rect) {
        let chain_info = &app.chain_info;
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Chain Info ");

        let Some(info) = &chain_info.data else {
            let msg = chain_info
                .error
                .as_deref()
                .unwrap_or("No Bitcoin node found. Is bitcoind running?");
            let p = Paragraph::new(msg)
                .style(Style::default().fg(Color::Yellow))
                .block(block)
                .wrap(Wrap { trim: true });
            f.render_widget(p, area);
            return;
        };

        let sync_pct = format!("{:.4}%", info.verificationprogress * 100.0);
        let ibd = if info.initialblockdownload {
            "yes"
        } else {
            "no"
        };
        let pruned = if info.pruned { "yes" } else { "no" };

        let rows = vec![
            Row::new(vec![
                Cell::from("Chain").style(Style::default().fg(Color::DarkGray)),
                Cell::from(info.chain.clone()),
            ]),
            Row::new(vec![
                Cell::from("Blocks").style(Style::default().fg(Color::DarkGray)),
                Cell::from(info.blocks.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Headers").style(Style::default().fg(Color::DarkGray)),
                Cell::from(info.headers.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Best Block").style(Style::default().fg(Color::DarkGray)),
                Cell::from(info.bestblockhash.clone()),
            ]),
            Row::new(vec![
                Cell::from("Difficulty").style(Style::default().fg(Color::DarkGray)),
                Cell::from(format!("{:.2}", info.difficulty)),
            ]),
            Row::new(vec![
                Cell::from("Time").style(Style::default().fg(Color::DarkGray)),
                Cell::from(info.time.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Median Time").style(Style::default().fg(Color::DarkGray)),
                Cell::from(info.mediantime.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Sync Progress").style(Style::default().fg(Color::DarkGray)),
                Cell::from(sync_pct),
            ]),
            Row::new(vec![
                Cell::from("Initial Block Download").style(Style::default().fg(Color::DarkGray)),
                Cell::from(ibd),
            ]),
            Row::new(vec![
                Cell::from("Chain Work").style(Style::default().fg(Color::DarkGray)),
                Cell::from(info.chainwork.clone()),
            ]),
            Row::new(vec![
                Cell::from("Size on Disk").style(Style::default().fg(Color::DarkGray)),
                Cell::from(format_size(info.size_on_disk)),
            ]),
            Row::new(vec![
                Cell::from("Pruned").style(Style::default().fg(Color::DarkGray)),
                Cell::from(pruned),
            ]),
            Row::new(vec![
                Cell::from("Warnings").style(Style::default().fg(Color::DarkGray)),
                Cell::from(if info.warnings.is_empty() {
                    "none".to_string()
                } else {
                    info.warnings.clone()
                }),
            ]),
        ];

        let widths = [Constraint::Length(24), Constraint::Min(0)];

        let table = Table::new(rows, widths)
            .block(block)
            .row_highlight_style(Style::default());

        f.render_widget(table, area);
    }
}

impl Default for ChainInfo {
    fn default() -> Self {
        Self::new()
    }
}
