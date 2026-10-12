//! Columnar named attributes ([E3D:A1] concept port): closed enum Column, no dyn/RTTI.
//! row = element index, column = named attribute; "missing" is semantically separate from zero values (Option sparse cells).
//! CORE-11/12/13 contract behavior.

#[derive(Clone, Debug, PartialEq)]
enum Column {
    F64(Vec<Option<f64>>),
    Str(Vec<Option<String>>),
    Bool(Vec<Option<bool>>),
}

impl Column {
    fn grow(&mut self, rows: usize) {
        match self {
            Self::F64(v) => v.resize(rows, None),
            Self::Str(v) => v.resize(rows, None),
            Self::Bool(v) => v.resize(rows, None),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Col {
    name: String,
    data: Column,
}

/// Attribute column set: row count aligns 1:1 with the host (e.g. `GeoDocument::features`) (CORE-12).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AttrSet {
    rows: usize,
    cols: Vec<Col>,
}

impl AttrSet {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Row count (column-store invariant: all columns are equally long == rows).
    #[must_use]
    pub fn len(&self) -> usize {
        self.rows
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows == 0
    }

    /// Append an empty row, return the row number.
    pub fn add_row(&mut self) -> usize {
        let r = self.rows;
        self.rows += 1;
        for c in &mut self.cols {
            c.data.grow(self.rows);
        }
        r
    }

    fn pad_to(&mut self, row: usize) {
        if row >= self.rows {
            self.rows = row + 1;
            for c in &mut self.cols {
                c.data.grow(self.rows);
            }
        }
    }

    fn find(&self, name: &str) -> Option<&Column> {
        self.cols.iter().find(|c| c.name == name).map(|c| &c.data)
    }

    /// f64 column write. Same-name different-type writes are rejected (CORE-13 first-write fixes the type).
    pub fn set_f64(&mut self, row: usize, name: &str, v: f64) -> bool {
        self.pad_to(row);
        match self.cols.iter_mut().find(|c| c.name == name) {
            Some(c) => match &mut c.data {
                Column::F64(d) => {
                    d[row] = Some(v);
                    true
                }
                _ => false,
            },
            None => {
                let mut d = vec![None; self.rows];
                d[row] = Some(v);
                self.cols.push(Col {
                    name: name.to_string(),
                    data: Column::F64(d),
                });
                true
            }
        }
    }

    /// String column write (one of the CORE-11 three types).
    pub fn set_str(&mut self, row: usize, name: &str, v: impl Into<String>) -> bool {
        self.pad_to(row);
        match self.cols.iter_mut().find(|c| c.name == name) {
            Some(c) => match &mut c.data {
                Column::Str(d) => {
                    d[row] = Some(v.into());
                    true
                }
                _ => false,
            },
            None => {
                let mut d: Vec<Option<String>> = vec![None; self.rows];
                d[row] = Some(v.into());
                self.cols.push(Col {
                    name: name.to_string(),
                    data: Column::Str(d),
                });
                true
            }
        }
    }

    /// bool column write.
    pub fn set_bool(&mut self, row: usize, name: &str, v: bool) -> bool {
        self.pad_to(row);
        match self.cols.iter_mut().find(|c| c.name == name) {
            Some(c) => match &mut c.data {
                Column::Bool(d) => {
                    d[row] = Some(v);
                    true
                }
                _ => false,
            },
            None => {
                let mut d = vec![None; self.rows];
                d[row] = Some(v);
                self.cols.push(Col {
                    name: name.to_string(),
                    data: Column::Bool(d),
                });
                true
            }
        }
    }

    /// Absence (column missing / type mismatch / row out of bounds / empty cell) is always None (CORE-11).
    #[must_use]
    pub fn f64(&self, row: usize, name: &str) -> Option<f64> {
        if row >= self.rows {
            return None;
        }
        match self.find(name) {
            Some(Column::F64(d)) => d[row],
            _ => None,
        }
    }

    #[must_use]
    pub fn str_value(&self, row: usize, name: &str) -> Option<&str> {
        if row >= self.rows {
            return None;
        }
        match self.find(name) {
            Some(Column::Str(d)) => d[row].as_deref(),
            _ => None,
        }
    }

    #[must_use]
    pub fn bool(&self, row: usize, name: &str) -> Option<bool> {
        if row >= self.rows {
            return None;
        }
        match self.find(name) {
            Some(Column::Bool(d)) => d[row],
            _ => None,
        }
    }

    /// Column-name enumeration (host introspection/debug).
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.cols.iter().map(|c| c.name.as_str())
    }
}
