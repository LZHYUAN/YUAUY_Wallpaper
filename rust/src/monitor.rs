//! Platform-free display topology records and the preserved C# area-weighted
//! monitor-selection arithmetic. Does not enumerate Windows or draw images.
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MonitorId(String);
impl MonitorId {
    pub fn new(value: impl Into<String>) -> Result<Self, MonitorError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(MonitorError::InvalidIdentifier);
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonitorBounds {
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
}
impl MonitorBounds {
    pub fn area(self) -> u64 {
        u64::from(self.width) * u64::from(self.height)
    }
    pub fn center(self) -> (i64, i64) {
        // Width and height are positive integers in the legacy C# source.
        (
            i64::from(self.left) + i64::from(self.width / 2),
            i64::from(self.top) + i64::from(self.height / 2),
        )
    }
    pub fn right(self) -> i64 {
        i64::from(self.left) + i64::from(self.width)
    }
    pub fn bottom(self) -> i64 {
        i64::from(self.top) + i64::from(self.height)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceLifetime {
    Unbound,
    Attached { generation: u64 },
    Lost { generation: u64 },
    Released,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorRecord {
    pub id: MonitorId,
    pub bounds: MonitorBounds,
    pub surface: SurfaceLifetime,
}
impl MonitorRecord {
    pub fn new(id: MonitorId, bounds: MonitorBounds) -> Self {
        Self {
            id,
            bounds,
            surface: SurfaceLifetime::Unbound,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorError {
    EmptyTopology,
    InvalidIdentifier,
    InvalidGeometry,
    DuplicateIdentifier,
    InvalidDraw,
    SelectionUnavailable,
}

#[derive(Debug, Clone)]
pub struct MonitorTopology {
    records: Vec<MonitorRecord>,
}
impl MonitorTopology {
    pub fn new(records: Vec<MonitorRecord>) -> Result<Self, MonitorError> {
        if records.is_empty() {
            return Err(MonitorError::EmptyTopology);
        }
        let mut names = HashSet::new();
        for record in &records {
            if record.bounds.width == 0 || record.bounds.height == 0 {
                return Err(MonitorError::InvalidGeometry);
            }
            if !names.insert(record.id.as_str()) {
                return Err(MonitorError::DuplicateIdentifier);
            }
        }
        Ok(Self { records })
    }
    pub fn records(&self) -> &[MonitorRecord] {
        &self.records
    }
    pub fn records_mut(&mut self) -> &mut [MonitorRecord] {
        &mut self.records
    }

    /// Returns the virtual desktop extent (left,top,right,bottom). Use wide
    /// intermediate integers so displays left/above primary remain representable.
    pub fn virtual_bounds(&self) -> (i64, i64, i64, i64) {
        let mut left = i64::MAX;
        let mut top = i64::MAX;
        let mut right = i64::MIN;
        let mut bottom = i64::MIN;
        for m in &self.records {
            left = left.min(i64::from(m.bounds.left));
            top = top.min(i64::from(m.bounds.top));
            right = right.max(m.bounds.right());
            bottom = bottom.max(m.bounds.bottom());
        }
        (left, top, right, bottom)
    }

    /// Preserve normal C# behavior: area=width*height in double, normalize by
    /// total, subtract each monitor's weight from NextDouble, break on <=0.
    /// The draw is supplied by the caller to keep randomness/testability separate.
    pub fn choose_index(&self, draw: f64) -> Result<usize, MonitorError> {
        if !draw.is_finite() || !(0.0..1.0).contains(&draw) {
            return Err(MonitorError::InvalidDraw);
        }
        let total = self
            .records
            .iter()
            .map(|m| m.bounds.area() as f64)
            .sum::<f64>();
        let mut remaining = draw;
        for (i, m) in self.records.iter().enumerate() {
            remaining -= (m.bounds.area() as f64) / total;
            if remaining <= 0.0 {
                return Ok(i);
            }
        }
        // The historical C# code would index past AllScreens here if the sum
        // rounded below draw. Report explicitly rather than crash or skew weight.
        Err(MonitorError::SelectionUnavailable)
    }
}
