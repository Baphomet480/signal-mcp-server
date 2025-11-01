use anyhow::{Context, Result};
use serde_json::Value;
use std::process::Stdio;
use tokio::process::Command;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct SignalCli {
    pub executable: std::path::PathBuf,
    pub account: String,
}

#[derive(Debug, Clone)]
pub struct ChatEntry {
    pub id: String,
    pub name: Option<String>,
}

impl SignalCli {
    pub fn new(executable: std::path::PathBuf, account: String) -> Self {
        Self {
            executable,
            account,
        }
    }

    pub async fn list_chats(&self) -> Result<Vec<ChatEntry>> {
        let mut chats = Vec::new();

        let contacts_output = Command::new(&self.executable)
            .arg("--account")
            .arg(&self.account)
            .arg("-o")
            .arg("json")
            .arg("listContacts")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .with_context(|| "failed to execute signal-cli listContacts")?;

        if !contacts_output.status.success() {
            let stderr = String::from_utf8_lossy(&contacts_output.stderr);
            return Err(anyhow::anyhow!(
                "signal-cli listContacts failed: {}",
                stderr.trim()
            ));
        }

        let contacts: Vec<Value> = serde_json::from_slice(&contacts_output.stdout)
            .with_context(|| "failed to parse signal-cli listContacts response")?;

        for contact in contacts {
            if let Some(number) = contact.get("number").and_then(Value::as_str) {
                let raw_name = contact.get("name").and_then(Value::as_str);
                let name = raw_name
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string());

                chats.push(ChatEntry {
                    id: number.to_string(),
                    name,
                });
            }
        }

        let groups_output = Command::new(&self.executable)
            .arg("--account")
            .arg(&self.account)
            .arg("-o")
            .arg("json")
            .arg("listGroups")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .with_context(|| "failed to execute signal-cli listGroups")?;

        if !groups_output.status.success() {
            let stderr = String::from_utf8_lossy(&groups_output.stderr);
            return Err(anyhow::anyhow!(
                "signal-cli listGroups failed: {}",
                stderr.trim()
            ));
        }

        let groups: Vec<Value> = serde_json::from_slice(&groups_output.stdout)
            .with_context(|| "failed to parse signal-cli listGroups response")?;

        for group in groups {
            if let Some(id) = group.get("id").and_then(Value::as_str) {
                let display_name = group
                    .get("name")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| id.to_string());

                chats.push(ChatEntry {
                    id: id.to_string(),
                    name: Some(display_name),
                });
            }
        }

        debug!(count = chats.len(), "signal-cli contacts/groups listed");
        Ok(chats)
    }

    pub async fn send_message(&self, recipient: &str, message: &str) -> Result<String> {
        let output = Command::new(&self.executable)
            .arg("--account")
            .arg(&self.account)
            .arg("send")
            .arg("-m")
            .arg(message)
            .arg(recipient)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .with_context(|| "failed to execute signal-cli send")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("signal-cli send failed: {}", stderr.trim()));
        }

        let response = String::from_utf8_lossy(&output.stdout).trim().to_string();
        debug!(?recipient, "signal-cli send succeeded");
        Ok(response)
    }
}
