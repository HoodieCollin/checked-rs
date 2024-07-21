use std::ops::RangeInclusive;

use proc_macro2::Span;

use crate::params::{NumberKind, NumberValue, NumberValueRange};

#[derive(Debug, Clone)]
pub struct RangeSeq {
    kind: NumberKind,
    has_full_range: bool,
    ranges: Vec<RangeInclusive<NumberValue>>,
}

impl RangeSeq {
    pub fn new(kind: NumberKind) -> Self {
        Self {
            kind,
            has_full_range: false,
            ranges: Vec::new(),
        }
    }

    pub fn with_capacity(kind: NumberKind, capacity: usize) -> Self {
        Self {
            kind,
            has_full_range: false,
            ranges: Vec::with_capacity(capacity),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    #[must_use]
    pub fn insert(&mut self, range: impl Into<NumberValueRange>) -> syn::Result<()> {
        let range: NumberValueRange = range.into();

        if self.kind != range.kind() {
            return Err(syn::Error::new(
                Span::call_site(),
                "Cannot mix different number kinds",
            ));
        }

        if matches!(range, NumberValueRange::Full(_)) {
            if self.has_full_range {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "Cannot have more than one full range",
                ));
            }

            self.has_full_range = true;
            return Ok(());
        }

        let range = RangeInclusive::new(range.first_val(), range.last_val());
        let mut dst_index = None;
        let start = *range.start();
        let end = *range.end();

        for (i, existing_range) in self.ranges.iter().enumerate() {
            let existing_start = *existing_range.start();
            let existing_end = *existing_range.end();

            // check if the new range is before the existing range
            if end < existing_start {
                dst_index = Some(i);
                break;
            }

            // check if the range is after the existing range
            if start > existing_end {
                continue;
            }

            // check if the start is within the existing range
            if start >= existing_start && start <= existing_end {
                return Err(syn::Error::new(
                    Span::call_site(),
                    format!("Range overlaps with existing range\n  start: {:?}, existing_start: {:?}, existing_end: {:?}", start, existing_start, existing_end)
                ));
            }

            // check if the end is within the existing range
            if end >= existing_start && end <= existing_end {
                return Err(syn::Error::new(
                    Span::call_site(),
                    format!("Range overlaps with existing range\n  end: {:?}, existing_start: {:?}, existing_end: {:?}", end, existing_start, existing_end)
                ));
            }
        }

        if let Some(i) = dst_index {
            self.ranges.insert(i, range);
        } else {
            self.ranges.push(range);
        }

        Ok(())
    }

    pub fn all_ranges(&self) -> Vec<RangeInclusive<NumberValue>> {
        if self.has_full_range {
            let full_range = {
                let value_range = NumberValueRange::Full(self.kind);
                value_range.first_val()..=value_range.last_val()
            };

            std::iter::once(full_range)
                .chain(self.ranges.iter().cloned())
                .collect()
        } else {
            self.ranges.clone()
        }
    }

    pub fn uniq_ranges(&self) -> Vec<NumberValueRange> {
        if self.has_full_range {
            vec![NumberValueRange::Full(self.kind)]
        } else {
            self.ranges.iter().map(|range| range.into()).collect()
        }
    }

    pub fn has_full_range(&self) -> bool {
        self.has_full_range
    }

    pub fn has_gaps(&self) -> bool {
        if self.has_full_range {
            return false;
        }

        let mut prev_end: Option<NumberValue> = None;

        for range in &self.ranges {
            if let Some(prev_end) = prev_end {
                if *range.start() != prev_end.add_usize(1) {
                    return true;
                }
            }

            prev_end = Some(*range.end());
        }

        false
    }

    pub fn first_uniq_val(&self) -> Option<NumberValue> {
        self.uniq_ranges()
            .first()
            .map(|range| range.first_val().clone())
    }

    pub fn last_uniq_val(&self) -> Option<NumberValue> {
        self.uniq_ranges()
            .last()
            .map(|range| range.last_val().clone())
    }
}
