use find::find_dirs;
use serde::Deserialize;
use skim::prelude::SkimItemReader;
use skim::prelude::SkimOptionsBuilder;
use skim::Skim;
use std::env;
use std::fs;
use std::io::Cursor;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};

mod find;

#[derive(Deserialize)]
struct Config {
    sessions: Vec<SSHSession>,
}

#[derive(Deserialize)]
struct SSHSession {
    name: String,
    protocol: String,
    host: String,           // e.g. "root@host"
    remote_command: String, // e.g. "cd /var/www; nvim ."
    split: Option<SplitConfig>,
}

#[derive(Deserialize)]
struct SplitConfig {
    #[serde(rename = "type")]
    split_type: String, // "vs" for vertical (side-by-side) or "hs" for horizontal (stacked)
    command: String, // the command to run in the split pane
}

fn main() {
    // Get the user's HOME directory.
    let home = env::var("HOME").expect("Could not determine HOME directory");

    // Determine the selection either from command-line argument or via fzf.
    let args: Vec<String> = env::args().collect();
    let selection: String = if args.len() == 2 {
        args[1].clone()
    } else {
        // Get local directories from ~/work via "find".
        let work_dir = format!("{}/work", home);
        let local_dirs = find_dirs(&work_dir, 2);

        // Load SSH sessions from TOML config.
        let config_path = format!("{}/dotfiles/sshprojects.toml", home);
        let ssh_names = if Path::new(&config_path).exists() {
            let config_contents =
                fs::read_to_string(&config_path).expect("Failed to read SSH sessions config file");
            let config: Config =
                toml::from_str(&config_contents).expect("Failed to parse TOML config");
            config
                .sessions
                .iter()
                .map(|s| s.name.clone())
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        // Combine local directories and SSH session names.
        let mut combined = Vec::new();
        combined.extend(local_dirs);
        combined.extend(ssh_names);

        let input = combined.join("\n");
        let options = SkimOptionsBuilder::default()
            .header(Some("AWESOME PROJECT PICKER".to_string()))
            .reverse(true)
            .build()
            .unwrap();
        let item_reader = SkimItemReader::default();
        let items = item_reader.of_bufread(Cursor::new(input));

        let mut selected_item = String::new();
        if let Some(output) = Skim::run_with(&options, Some(items)) {
            for item in output.selected_items {
                selected_item = item.output().to_string();
            }
        }
        selected_item
    };

    if selection.is_empty() {
        return;
    }

    // Branch: if the selection is a local directory.
    if Path::new(&selection).is_dir() {
        // Use the basename (with dots replaced by underscores) as the tmux session name.
        let session_name = Path::new(&selection)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("session")
            .replace(".", "_");
        run_local_tmux_session(&session_name, &selection);
    } else {
        // Otherwise, assume it is an SSH session name.
        let config_path = format!("{}/dotfiles/sshprojects.toml", home);
        let config_contents =
            fs::read_to_string(&config_path).expect("Failed to read SSH sessions config file");
        let config: Config = toml::from_str(&config_contents).expect("Failed to parse TOML config");
        let session_config = config
            .sessions
            .iter()
            .find(|s| s.name == selection)
            .expect("SSH session not found in config");
        if session_config.protocol != "ssh" {
            eprintln!(
                "Invalid protocol '{}' for session '{}'",
                session_config.protocol, session_config.name
            );
            return;
        }
        run_ssh_tmux_session(session_config);
    }
}

/// Launches a tmux session for a local directory.
fn run_local_tmux_session(session_name: &str, directory: &str) {
    let in_tmux = env::var("TMUX").is_ok();
    let tmux_running = Command::new("pgrep")
        .arg("tmux")
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if !in_tmux && !tmux_running {
        // Not inside tmux and tmux is not running.
        Command::new("tmux")
            .args(&["new-session", "-s", session_name, "-c", directory])
            .status()
            .expect("Failed to create tmux session");
        return;
    }

    // If the session does not exist, create it in detached mode.
    let session_exists = Command::new("tmux")
        .args(&["has-session", "-t", session_name])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !session_exists {
        Command::new("tmux")
            .args(&["new-session", "-ds", session_name, "-c", directory])
            .status()
            .expect("Failed to create detached tmux session");
    }

    // Attach or switch to the tmux session.
    if !in_tmux {
        Command::new("tmux")
            .args(&["attach", "-t", session_name])
            .status()
            .expect("Failed to attach to tmux session");
    } else {
        Command::new("tmux")
            .args(&["switch-client", "-t", session_name])
            .status()
            .expect("Failed to switch tmux client");
    }
}

/// Launches a tmux session for an SSH session.
fn run_ssh_tmux_session(session_config: &SSHSession) {
    let session_name = &session_config.name;
    let host = &session_config.host;
    let remote_command = &session_config.remote_command;
    let in_tmux = env::var("TMUX").is_ok();
    let tmux_running = Command::new("pgrep")
        .arg("tmux")
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    // Create the session if needed.
    if !in_tmux && !tmux_running {
        Command::new("tmux")
            .args(&["new-session", "-s", session_name, "-d", "-n", "main"])
            .status()
            .expect("Failed to create tmux session");
        let ssh_cmd = format!("ssh -t {} '{}; exec $SHELL'", host, remote_command);
        Command::new("tmux")
            .args(&[
                "send-keys",
                "-t",
                &format!("{}:main", session_name),
                &ssh_cmd,
                "C-m",
            ])
            .status()
            .expect("Failed to send keys to tmux session");
        if let Some(split) = &session_config.split {
            create_tmux_split(session_name, host, split);
        }
        Command::new("tmux")
            .args(&["attach", "-t", session_name])
            .status()
            .expect("Failed to attach to tmux session");
        return;
    }

    let session_exists = Command::new("tmux")
        .args(&["has-session", "-t", session_name])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !session_exists {
        Command::new("tmux")
            .args(&["new-session", "-ds", session_name, "-n", "main"])
            .status()
            .expect("Failed to create detached tmux session");
        let ssh_cmd = format!("ssh -t {} '{}; exec $SHELL'", host, remote_command);
        Command::new("tmux")
            .args(&[
                "send-keys",
                "-t",
                &format!("{}:main", session_name),
                &ssh_cmd,
                "C-m",
            ])
            .status()
            .expect("Failed to send keys to tmux session");
    }

    if let Some(split) = &session_config.split {
        create_tmux_split(session_name, host, split);
    }

    if !in_tmux {
        Command::new("tmux")
            .args(&["attach", "-t", session_name])
            .status()
            .expect("Failed to attach to tmux session");
    } else {
        Command::new("tmux")
            .args(&["switch-client", "-t", session_name])
            .status()
            .expect("Failed to switch tmux client");
    }
}

/// Creates a split pane in the given tmux session. The split occupies 20% of the main window.
/// After creating the split and sending the SSH command, the focus returns to the main pane.
fn create_tmux_split(session_name: &str, host: &str, split: &SplitConfig) {
    let split_opt = match split.split_type.as_str() {
        "vs" => "-h", // vertical split
        "hs" => "-v", // horizontal split
        _ => "-h",
    };

    // Get the current (main) pane ID.
    let main_pane_output = Command::new("tmux")
        .args(&["display-message", "-p", "#{pane_id}"])
        .output()
        .expect("Failed to get current pane id");
    let main_pane = String::from_utf8_lossy(&main_pane_output.stdout)
        .trim()
        .to_string();

    // Create the split (20% of the window) in the main window.
    Command::new("tmux")
        .args(&[
            "split-window",
            split_opt,
            "-p",
            "20",
            "-t",
            &format!("{}:main", session_name),
        ])
        .status()
        .expect("Failed to create tmux split");

    // Get the new pane's ID (assumed to be the last pane).
    let list_output = Command::new("tmux")
        .args(&[
            "list-panes",
            "-t",
            &format!("{}:main", session_name),
            "-F",
            "#{pane_id}",
        ])
        .output()
        .expect("Failed to list tmux panes");
    let list_output = String::from_utf8_lossy(&list_output.stdout);
    let panes: Vec<&str> = list_output.lines().collect();
    let new_pane = panes.last().expect("No pane found").to_string();

    // Send the SSH command to the new pane.
    let ssh_cmd = format!("ssh -t {} '{}; exec $SHELL'", host, split.command);
    Command::new("tmux")
        .args(&["send-keys", "-t", &new_pane, &ssh_cmd, "C-m"])
        .status()
        .expect("Failed to send keys to split pane");

    // Return focus to the main pane.
    Command::new("tmux")
        .args(&["select-pane", "-t", &main_pane])
        .status()
        .expect("Failed to select main pane");
}
