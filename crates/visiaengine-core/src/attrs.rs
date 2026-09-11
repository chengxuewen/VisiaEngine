//! 列式命名属性（[E3D:A1] 概念移植）：闭合 enum Column，无 dyn/RTTI。
//! 行=元素索引、列=命名属性；"缺失"与零值语义分离（Option 稀疏格）。
//! CORE-11/12/13 契约行为。

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

/// 属性列集：行数与宿主（如 `GeoDocument::features`）1:1 对齐（CORE-12）。
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

    /// 行数（列存储不变式：所有列等长 == rows）。
    #[must_use]
    pub fn len(&self) -> usize {
        self.rows
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows == 0
    }

    /// 追加空行，返回行号。
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

    /// f64 列写入。同名异型写拒绝（CORE-13 首写定型）。
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

    /// 字符串列写入（CORE-11 三型之一）。
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

    /// bool 列写入。
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

    /// 缺失（列不存在/异型/行越界/空格）一律 None（CORE-11）。
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

    /// 列名枚举（宿主自省/调试）。
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.cols.iter().map(|c| c.name.as_str())
    }
}
