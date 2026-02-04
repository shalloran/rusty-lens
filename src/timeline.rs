// defender timeline event: one row of the 66-column csv

use chrono::{Duration, Local, NaiveDateTime};
use serde::Deserialize;

/// parse relative time range from string; `now` is reference (e.g. Local::now().naive_local()).
/// returns (start, end) inclusive; e.g. "today" -> (start_of_today, end_of_today).
pub fn parse_relative_range(
    s: &str,
    now: NaiveDateTime,
) -> Option<(Option<NaiveDateTime>, Option<NaiveDateTime>)> {
    let s = s.trim().to_lowercase();
    if s.is_empty() {
        return None;
    }
    let date = now.date();
    let start_of_today = date.and_hms_opt(0, 0, 0).unwrap();
    let end_of_today = date.and_hms_opt(23, 59, 59).unwrap();
    match s.as_str() {
        "today" => Some((Some(start_of_today), Some(end_of_today))),
        "yesterday" => {
            let y = date.pred_opt()?;
            Some((
                Some(y.and_hms_opt(0, 0, 0).unwrap()),
                Some(y.and_hms_opt(23, 59, 59).unwrap()),
            ))
        }
        "last 24 hours" | "last 24h" | "24h" => {
            let start = now - Duration::hours(24);
            Some((Some(start), Some(now)))
        }
        "last 7 days" | "last 7d" | "7d" => {
            let start = (now - Duration::days(7))
                .date()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            Some((Some(start), Some(now)))
        }
        "last 30 days" | "last 30d" | "30d" => {
            let start = (now - Duration::days(30))
                .date()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            Some((Some(start), Some(now)))
        }
        "last 1 hour" | "last 1h" | "1h" => {
            let start = now - Duration::hours(1);
            Some((Some(start), Some(now)))
        }
        "last 12 hours" | "last 12h" | "12h" => {
            let start = now - Duration::hours(12);
            Some((Some(start), Some(now)))
        }
        _ => None,
    }
}

/// reference "now" for relative times (local time)
pub fn now_for_relative() -> NaiveDateTime {
    Local::now().naive_local()
}

/// parse iso-like timestamp (event time or user input); tries a few formats
pub fn parse_time(s: &str) -> Option<NaiveDateTime> {
    let s = s.trim().trim_matches('"').trim();
    if s.is_empty() {
        return None;
    }
    const FORMATS: &[&str] = &[
        "%Y-%m-%dT%H:%M:%S%.3f",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S%.3f",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%d",
    ];
    for fmt in FORMATS {
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, fmt) {
            return Some(dt);
        }
    }
    // allow date without time -> start of day
    if s.len() >= 10 {
        if let Ok(d) = chrono::NaiveDate::parse_from_str(&s[..10], "%Y-%m-%d") {
            return Some(d.and_hms_opt(0, 0, 0).unwrap());
        }
    }
    None
}

/// one device timeline event (66 columns); empty csv cells deserialize as None
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct TimelineEvent {
    #[serde(rename = "Event Time")]
    pub event_time: Option<String>,
    #[serde(rename = "Machine Id")]
    pub machine_id: Option<String>,
    #[serde(rename = "Computer Name")]
    pub computer_name: Option<String>,
    #[serde(rename = "Action Type")]
    pub action_type: Option<String>,
    #[serde(rename = "File Name")]
    pub file_name: Option<String>,
    #[serde(rename = "Folder Path")]
    pub folder_path: Option<String>,
    #[serde(rename = "Sha1")]
    pub sha1: Option<String>,
    #[serde(rename = "Sha256")]
    pub sha256: Option<String>,
    #[serde(rename = "MD5")]
    pub md5: Option<String>,
    #[serde(rename = "Process Command Line")]
    pub process_command_line: Option<String>,
    #[serde(rename = "Account Domain")]
    pub account_domain: Option<String>,
    #[serde(rename = "Account Name")]
    pub account_name: Option<String>,
    #[serde(rename = "Account Sid")]
    pub account_sid: Option<String>,
    #[serde(rename = "Logon Id")]
    pub logon_id: Option<String>,
    #[serde(rename = "Process Id")]
    pub process_id: Option<String>,
    #[serde(rename = "Process Creation Time")]
    pub process_creation_time: Option<String>,
    #[serde(rename = "Process Token Elevation")]
    pub process_token_elevation: Option<String>,
    #[serde(rename = "Registry Key")]
    pub registry_key: Option<String>,
    #[serde(rename = "Registry Value Name")]
    pub registry_value_name: Option<String>,
    #[serde(rename = "Registry Value Data")]
    pub registry_value_data: Option<String>,
    #[serde(rename = "Remote Url")]
    pub remote_url: Option<String>,
    #[serde(rename = "Remote Computer Name")]
    pub remote_computer_name: Option<String>,
    #[serde(rename = "Remote IP")]
    pub remote_ip: Option<String>,
    #[serde(rename = "Remote Port")]
    pub remote_port: Option<String>,
    #[serde(rename = "Local IP")]
    pub local_ip: Option<String>,
    #[serde(rename = "Local Port")]
    pub local_port: Option<String>,
    #[serde(rename = "File Origin Url")]
    pub file_origin_url: Option<String>,
    #[serde(rename = "File Origin IP")]
    pub file_origin_ip: Option<String>,
    #[serde(rename = "Initiating Process SHA1")]
    pub initiating_process_sha1: Option<String>,
    #[serde(rename = "Initiating Process SHA256")]
    pub initiating_process_sha256: Option<String>,
    #[serde(rename = "Initiating Process File Name")]
    pub initiating_process_file_name: Option<String>,
    #[serde(rename = "Initiating Process Folder Path")]
    pub initiating_process_folder_path: Option<String>,
    #[serde(rename = "Initiating Process Id")]
    pub initiating_process_id: Option<String>,
    #[serde(rename = "Initiating Process Command Line")]
    pub initiating_process_command_line: Option<String>,
    #[serde(rename = "Initiating Process Creation Time")]
    pub initiating_process_creation_time: Option<String>,
    #[serde(rename = "Initiating Process Integrity Level")]
    pub initiating_process_integrity_level: Option<String>,
    #[serde(rename = "Initiating Process Token Elevation")]
    pub initiating_process_token_elevation: Option<String>,
    #[serde(rename = "Initiating Process Parent Id")]
    pub initiating_process_parent_id: Option<String>,
    #[serde(rename = "Initiating Process Parent File Name")]
    pub initiating_process_parent_file_name: Option<String>,
    #[serde(rename = "Initiating Process Parent Creation Time")]
    pub initiating_process_parent_creation_time: Option<String>,
    #[serde(rename = "Initiating Process MD5")]
    pub initiating_process_md5: Option<String>,
    #[serde(rename = "Initiating Process Account Domain")]
    pub initiating_process_account_domain: Option<String>,
    #[serde(rename = "Initiating Process Account Name")]
    pub initiating_process_account_name: Option<String>,
    #[serde(rename = "Initiating Process Account Sid")]
    pub initiating_process_account_sid: Option<String>,
    #[serde(rename = "Initiating Process Logon Id")]
    pub initiating_process_logon_id: Option<String>,
    #[serde(rename = "Report Id")]
    pub report_id: Option<String>,
    #[serde(rename = "Additional Fields")]
    pub additional_fields: Option<String>,
    #[serde(rename = "Typed Details")]
    pub typed_details: Option<String>,
    #[serde(rename = "App Guard Container Id")]
    pub app_guard_container_id: Option<String>,
    pub protocol: Option<String>,
    #[serde(rename = "Logon Type")]
    pub logon_type: Option<String>,
    #[serde(rename = "Process Integrity Level")]
    pub process_integrity_level: Option<String>,
    #[serde(rename = "Registry Value Type")]
    pub registry_value_type: Option<String>,
    #[serde(rename = "Previous Registry Value Name")]
    pub previous_registry_value_name: Option<String>,
    #[serde(rename = "Previous Registry Value Data")]
    pub previous_registry_value_data: Option<String>,
    #[serde(rename = "Previous Registry Key")]
    pub previous_registry_key: Option<String>,
    #[serde(rename = "File Origin Referrer Url")]
    pub file_origin_referrer_url: Option<String>,
    #[serde(rename = "Sensitivity Label")]
    pub sensitivity_label: Option<String>,
    #[serde(rename = "Sensitivity Sub Label")]
    pub sensitivity_sub_label: Option<String>,
    #[serde(rename = "Is Endpoint Dlp Applied")]
    pub is_endpoint_dlp_applied: Option<String>,
    #[serde(rename = "Is Azure Info Protection Applied")]
    pub is_azure_info_protection_applied: Option<String>,
    #[serde(rename = "Alert Ids")]
    pub alert_ids: Option<String>,
    pub categories: Option<String>,
    pub severities: Option<String>,
    #[serde(rename = "Is Marked")]
    pub is_marked: Option<String>,
    #[serde(rename = "Data Type")]
    pub data_type: Option<String>,
}

impl TimelineEvent {
    /// event time as parsed datetime (for range filtering)
    pub fn event_time_parsed(&self) -> Option<NaiveDateTime> {
        self.event_time.as_deref().and_then(parse_time)
    }

    /// true if event time falls within [start, end] (inclusive); missing start/end means no bound
    pub fn in_time_range(&self, start: Option<NaiveDateTime>, end: Option<NaiveDateTime>) -> bool {
        let t = match self.event_time_parsed() {
            Some(t) => t,
            None => return start.is_none() && end.is_none(),
        };
        if let Some(s) = start {
            if t < s {
                return false;
            }
        }
        if let Some(e) = end {
            if t > e {
                return false;
            }
        }
        true
    }

    /// short one-line summary for list view: time | action | file or process
    pub fn list_line(&self) -> String {
        let time = self.event_time.as_deref().unwrap_or("").trim_matches('"');
        let action = self.action_type.as_deref().unwrap_or("—");
        let file = self
            .file_name
            .as_deref()
            .or(self.initiating_process_file_name.as_deref())
            .unwrap_or("")
            .trim_matches('"');
        let computer = self
            .computer_name
            .as_deref()
            .unwrap_or("")
            .trim_matches('"');
        if file.is_empty() {
            format!("{} | {} | {}", time, action, computer)
        } else {
            format!("{} | {} | {}", time, action, file)
        }
    }

    /// all non-empty fields for detail view (label: value)
    pub fn detail_lines(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        let mut push = |label: &str, v: Option<&String>| {
            if let Some(s) = v {
                let s = s.trim_matches('"').trim();
                if !s.is_empty() {
                    out.push((label.to_string(), s.to_string()));
                }
            }
        };
        push("Event Time", self.event_time.as_ref());
        push("Machine Id", self.machine_id.as_ref());
        push("Computer Name", self.computer_name.as_ref());
        push("Action Type", self.action_type.as_ref());
        push("File Name", self.file_name.as_ref());
        push("Folder Path", self.folder_path.as_ref());
        push("Sha1", self.sha1.as_ref());
        push("Sha256", self.sha256.as_ref());
        push("MD5", self.md5.as_ref());
        push("Process Command Line", self.process_command_line.as_ref());
        push("Account Domain", self.account_domain.as_ref());
        push("Account Name", self.account_name.as_ref());
        push("Account Sid", self.account_sid.as_ref());
        push("Logon Id", self.logon_id.as_ref());
        push("Process Id", self.process_id.as_ref());
        push("Process Creation Time", self.process_creation_time.as_ref());
        push(
            "Process Token Elevation",
            self.process_token_elevation.as_ref(),
        );
        push("Registry Key", self.registry_key.as_ref());
        push("Registry Value Name", self.registry_value_name.as_ref());
        push("Registry Value Data", self.registry_value_data.as_ref());
        push("Remote Url", self.remote_url.as_ref());
        push("Remote Computer Name", self.remote_computer_name.as_ref());
        push("Remote IP", self.remote_ip.as_ref());
        push("Remote Port", self.remote_port.as_ref());
        push("Local IP", self.local_ip.as_ref());
        push("Local Port", self.local_port.as_ref());
        push("File Origin Url", self.file_origin_url.as_ref());
        push("File Origin IP", self.file_origin_ip.as_ref());
        push(
            "Initiating Process SHA1",
            self.initiating_process_sha1.as_ref(),
        );
        push(
            "Initiating Process SHA256",
            self.initiating_process_sha256.as_ref(),
        );
        push(
            "Initiating Process File Name",
            self.initiating_process_file_name.as_ref(),
        );
        push(
            "Initiating Process Folder Path",
            self.initiating_process_folder_path.as_ref(),
        );
        push("Initiating Process Id", self.initiating_process_id.as_ref());
        push(
            "Initiating Process Command Line",
            self.initiating_process_command_line.as_ref(),
        );
        push(
            "Initiating Process Creation Time",
            self.initiating_process_creation_time.as_ref(),
        );
        push(
            "Initiating Process Integrity Level",
            self.initiating_process_integrity_level.as_ref(),
        );
        push(
            "Initiating Process Token Elevation",
            self.initiating_process_token_elevation.as_ref(),
        );
        push(
            "Initiating Process Parent Id",
            self.initiating_process_parent_id.as_ref(),
        );
        push(
            "Initiating Process Parent File Name",
            self.initiating_process_parent_file_name.as_ref(),
        );
        push(
            "Initiating Process Parent Creation Time",
            self.initiating_process_parent_creation_time.as_ref(),
        );
        push(
            "Initiating Process MD5",
            self.initiating_process_md5.as_ref(),
        );
        push(
            "Initiating Process Account Domain",
            self.initiating_process_account_domain.as_ref(),
        );
        push(
            "Initiating Process Account Name",
            self.initiating_process_account_name.as_ref(),
        );
        push(
            "Initiating Process Account Sid",
            self.initiating_process_account_sid.as_ref(),
        );
        push(
            "Initiating Process Logon Id",
            self.initiating_process_logon_id.as_ref(),
        );
        push("Report Id", self.report_id.as_ref());
        push("Additional Fields", self.additional_fields.as_ref());
        push("Typed Details", self.typed_details.as_ref());
        push(
            "App Guard Container Id",
            self.app_guard_container_id.as_ref(),
        );
        push("Protocol", self.protocol.as_ref());
        push("Logon Type", self.logon_type.as_ref());
        push(
            "Process Integrity Level",
            self.process_integrity_level.as_ref(),
        );
        push("Registry Value Type", self.registry_value_type.as_ref());
        push(
            "Previous Registry Value Name",
            self.previous_registry_value_name.as_ref(),
        );
        push(
            "Previous Registry Value Data",
            self.previous_registry_value_data.as_ref(),
        );
        push("Previous Registry Key", self.previous_registry_key.as_ref());
        push(
            "File Origin Referrer Url",
            self.file_origin_referrer_url.as_ref(),
        );
        push("Sensitivity Label", self.sensitivity_label.as_ref());
        push("Sensitivity Sub Label", self.sensitivity_sub_label.as_ref());
        push(
            "Is Endpoint Dlp Applied",
            self.is_endpoint_dlp_applied.as_ref(),
        );
        push(
            "Is Azure Info Protection Applied",
            self.is_azure_info_protection_applied.as_ref(),
        );
        push("Alert Ids", self.alert_ids.as_ref());
        push("Categories", self.categories.as_ref());
        push("Severities", self.severities.as_ref());
        push("Is Marked", self.is_marked.as_ref());
        push("Data Type", self.data_type.as_ref());
        out
    }

    /// concatenation of all searchable text (one string for multi-token matching)
    fn searchable_text(&self) -> String {
        let mut out = String::new();
        let mut push = |s: Option<&String>| {
            if let Some(x) = s {
                out.push_str(&x.to_lowercase());
                out.push(' ');
            }
        };
        push(self.event_time.as_ref());
        push(self.machine_id.as_ref());
        push(self.computer_name.as_ref());
        push(self.action_type.as_ref());
        push(self.file_name.as_ref());
        push(self.folder_path.as_ref());
        push(self.sha1.as_ref());
        push(self.sha256.as_ref());
        push(self.md5.as_ref());
        push(self.process_command_line.as_ref());
        push(self.account_domain.as_ref());
        push(self.account_name.as_ref());
        push(self.account_sid.as_ref());
        push(self.process_id.as_ref());
        push(self.process_creation_time.as_ref());
        push(self.registry_key.as_ref());
        push(self.registry_value_name.as_ref());
        push(self.registry_value_data.as_ref());
        push(self.remote_url.as_ref());
        push(self.remote_computer_name.as_ref());
        push(self.remote_ip.as_ref());
        push(self.remote_port.as_ref());
        push(self.local_ip.as_ref());
        push(self.local_port.as_ref());
        push(self.file_origin_url.as_ref());
        push(self.file_origin_ip.as_ref());
        push(self.initiating_process_sha1.as_ref());
        push(self.initiating_process_sha256.as_ref());
        push(self.initiating_process_file_name.as_ref());
        push(self.initiating_process_folder_path.as_ref());
        push(self.initiating_process_id.as_ref());
        push(self.initiating_process_command_line.as_ref());
        push(self.initiating_process_creation_time.as_ref());
        push(self.initiating_process_parent_file_name.as_ref());
        push(self.initiating_process_account_domain.as_ref());
        push(self.initiating_process_account_name.as_ref());
        push(self.report_id.as_ref());
        push(self.additional_fields.as_ref());
        push(self.typed_details.as_ref());
        push(self.protocol.as_ref());
        push(self.alert_ids.as_ref());
        push(self.categories.as_ref());
        push(self.severities.as_ref());
        push(self.data_type.as_ref());
        out
    }

    /// true if event matches `needle` (case-insensitive). empty needle = match all.
    /// multi-word: space-separated tokens are ANDed (all must appear in searchable fields).
    pub fn matches_search(&self, needle: &str) -> bool {
        let needle = needle.trim();
        if needle.is_empty() {
            return true;
        }
        let haystack = self.searchable_text();
        let tokens: Vec<String> = needle
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if tokens.is_empty() {
            return true;
        }
        tokens.iter().all(|t| haystack.contains(t.as_str()))
    }
}
