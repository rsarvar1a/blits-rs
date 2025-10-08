
use crate::prelude::*;

impl<'a> Board<'a> {
    /// Pretty-prints the board.
    pub fn pretty(&self) -> String {
        self.cells.0.iter().map(|row| {
            row.map(|cell| {
                format!("{}", cell)
            }).join("")
        }).collect::<Vec<String>>().join("\n")
    }
}
