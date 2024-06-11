use std::sync::OnceLock;

use regex::Regex;
use thiserror::Error;
use time::{error, Duration};

fn duration_unit_syntax_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        let mut r = Regex::new(r"^(\d+)(s|m|h|d|w)$").unwrap();
        r
    })
}

#[derive(Debug, Error)]
pub enum DurationSequenceError {
    #[error("Duration sequence is empty")]
    EmptySequence,
}

/// A duration sequence is a Vec<Duration> where it stores a interval of time where a event should happen. For example:
/// [5m, 5m, 1h, 12h, 36h, 1d, 1d, 3d] represents that a event should happen first in 5 minutes than 5 minutes than 1 hour and so on.
#[derive(Debug)]
pub struct DurationSequence {
    sequence: Vec<Duration>,
    total_duration: Duration,
}

impl DurationSequence {
    /// Create a instance of DurationSequence from a Vec<Duration>
    pub fn from_vec(dur_seq: Vec<Duration>) -> Result<DurationSequence, DurationSequenceError> {
        if dur_seq.len() == 0 {
            return Err(DurationSequenceError::EmptySequence);
        }

        let total_duration = dur_seq.iter().sum();
        return Ok(DurationSequence { sequence: dur_seq, total_duration })
    }

    /// Return a element from the duration sequence wrapped on a Option
    pub fn get_from_sequence(&self, index: usize) -> Option<&Duration> {
        return self.sequence.get(index);
    }

    /// Return a element from the sequence if its not found return the first element. This function asserts
    /// that this instance has at least 1 element
    pub fn get_from_sequence_or_first(&self, index: usize) -> &Duration {
        return self.get_from_sequence(index).unwrap_or_else(|| self.sequence.get(0).unwrap())
    }

    /// Add a duration into the duration sequence of this instance. Also return a mutable reference to this
    /// instance so it can call another function in cascade
    pub fn push(&mut self, d: Duration) -> &mut Self {
        self.sequence.push(d);
        self.total_duration += d;
        return self
    }

    /// Return the reference to the duration sequence of this instance
    pub fn sequence(&self) -> &Vec<Duration> {
        &self.sequence
    }

    /// Return the reference to the total duration of this instance
    pub fn total_duration(&self) -> &Duration {
        &self.total_duration
    }

}

/// DurationSequence implementation of Clone
impl Clone for DurationSequence {
    fn clone(&self) -> Self {
        Self { sequence: self.sequence.clone(), total_duration: self.total_duration.clone() }
    }
}

/// DurationSequence implementation of PartialEq
impl PartialEq for DurationSequence {
    fn eq(&self, other: &Self) -> bool {
        self.sequence == other.sequence && self.total_duration == other.total_duration
    }
}

#[derive(Debug, Error)]
pub enum DurationSerdeErrors {
    #[error("Data has invalid syntax")]
    InvalidSyntax,
}

pub trait DurationDeserializer {
    /// Tells a type how to deserialize a data into a Duration type
    fn to_duration(&self) -> Result<Duration, DurationSerdeErrors>;
}

pub trait DurationSequenceDeserializer {
    fn to_duration_sequence(&self) -> Result<DurationSequence, DurationSerdeErrors>;
}

/// Implement DurationDeserializer trait for String
impl DurationDeserializer for &str {
    fn to_duration(&self) -> Result<Duration, DurationSerdeErrors> {
        // Check if the regex matches and extract captures
        let captures = duration_unit_syntax_regex().captures(self);
        if let Some(caps) = captures {
            let number_str = caps.get(1).unwrap().as_str();
            let unit = caps.get(2).unwrap().as_str();
            let number: i64 = number_str.parse().map_err(|_| DurationSerdeErrors::InvalidSyntax)?;

            // Match the unit and create the corresponding Duration
            let duration = match unit {
                "s" => Duration::seconds(number),
                "m" => Duration::minutes(number),
                "h" => Duration::hours(number),
                "d" => Duration::days(number),
                "w" => Duration::weeks(number),
                _ => return Err(DurationSerdeErrors::InvalidSyntax), // should not happen
            };
            Ok(duration)
        } else {
            Err(DurationSerdeErrors::InvalidSyntax)
        }
    }
}

impl DurationSequenceDeserializer for &str {
    fn to_duration_sequence(&self) -> Result<DurationSequence, DurationSerdeErrors> {
        // if the value does is not contained by '['']' this compiler will assume that it is a single
        // value sequence (like 1d, 30m, etc,) so it will deserialize as self.to_duration and included it in a sequence
        if !self.starts_with('[')  || !self.ends_with(']') {
            return Ok(DurationSequence::from_vec(vec![self.to_duration()?]).unwrap())
        }

        // remove '[' ']'
        let normalized_str = self.trim_matches(&['[', ']'][..]);

        // extract all values splitted by ','
        let mut duration_seq = Vec::new();
        for dur_unit in normalized_str.split(',') {
            let normalized_dur_unit = dur_unit.trim();
            duration_seq.push(normalized_dur_unit.to_duration()?);
        }

        // assert that at least one value is passed in duration sequence vec
        if duration_seq.len() < 1 {
            return Err(DurationSerdeErrors::InvalidSyntax)
        }

        Ok(DurationSequence::from_vec(duration_seq).unwrap())
    }
}

#[cfg(test)]
mod tests {

    use std::vec;

    use super::*;

    /// Tests if DurationSequence::from_vec() has expected values
    #[test]
    fn test_if_duration_sequence_from_vec_has_expected_values() {
        let dur_seq_vec = vec![
            Duration::minutes(5), Duration::minutes(5), Duration::hours(1),  // same day / close range
            Duration::hours(12), Duration::hours(36), // same-day/other-day + 1 day
            Duration::days(1), Duration::days(1), Duration::days(1), Duration::days(3) // days range
            ];
        let dur_seq = DurationSequence::from_vec(dur_seq_vec).unwrap();

        assert_eq!(dur_seq.total_duration.whole_days(), 8);
        assert_eq!(dur_seq.total_duration.whole_hours(), (3 + 1 + 1 + 1) * 24 + (36 + 12 + 1));
    }

    #[test]
    #[should_panic]
    fn test_if_duration_sequence_panics_at_empty_sequence() {
        DurationSequence::from_vec(vec![]).unwrap();
    }

    #[test]
    fn test_if_string_to_duration_works() {
        let duration = "5d".to_duration().unwrap();
        assert_eq!(duration.whole_days(), 5)
    }

    #[test]
    #[should_panic]
    fn test_if_string_to_duration_panics_at_invalid_syntax() {
        "5x".to_duration().unwrap(); // should panic
    }

    #[test]
    fn test_if_string_to_duration_sequence_works() {
        let duration_seq = "[5m, 5m, 1h, 12h, 36h, 1d, 1d, 1d, 3d]".to_duration_sequence().unwrap();
        assert_eq!(duration_seq.total_duration.whole_days(), 8);
        assert_eq!(duration_seq.total_duration.whole_hours(), (3 + 1 + 1 + 1) * 24 + (36 + 12 + 1));
    }

    #[test]
    #[should_panic]
    fn test_if_string_to_duration_sequence_panics_at_invalid_missing_comma() {
        "[5m 5m, 1h, 12h, 36h, 1d, 1d, 1d, 3d]".to_duration_sequence().unwrap(); // should panic (missing ',')
    }

    
    #[test]
    #[should_panic]
    fn test_if_string_to_duration_sequence_panics_at_invalid_missing_square_bracket() {
        "5m, 5m, 1h, 12h, 36h, 1d, 1d, 1d, 3d".to_duration_sequence().unwrap(); // should panic (missing '[' ']')
    }

}