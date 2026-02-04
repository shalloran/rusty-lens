// streaming csv reader for defender timeline; collects rows into a vec (with optional cap)

use crate::error::Result;
use crate::timeline::TimelineEvent;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// load timeline events from a csv path; malformed rows are skipped
pub fn load_timeline(path: &Path, max_rows: Option<usize>) -> Result<Vec<TimelineEvent>> {
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(BufReader::new(file));
    let mut out = Vec::new();
    for row in rdr.deserialize() {
        if let Some(cap) = max_rows {
            if out.len() >= cap {
                break;
            }
        }
        if let Ok(ev) = row {
            out.push(ev);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn parse_fixture() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixture_sample.csv");
        let events = load_timeline(&path, None).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].action_type.as_deref(), Some("ProcessCreated"));
        assert_eq!(events[1].action_type.as_deref(), Some("ConnectionSuccess"));
    }
}
