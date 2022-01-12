use std::fmt::{self, Formatter};

#[derive(Debug, PartialEq)]
pub struct Cursor {
    before: String,
    after: String,
}

impl Cursor {
    pub fn new() -> Self {
        Cursor {
            before: String::new(),
            after: String::new(),
        }
    }

    pub fn from(before: String, after: String) -> Self {
        Cursor {
            before,
            after: after.chars().rev().collect(),
        }
    }

    pub fn pos(&self) -> usize {
        self.before.len() + 1
    }

    pub fn insert(&mut self, c: char) {
        self.before.push(c);
    }

    pub fn backspace(&mut self) {
        self.before.pop();
    }

    pub fn delete(&mut self) {
        self.after.pop();
    }

    /*
     * Normal mode movements
     */
    pub fn left_char(&mut self) {
        if let Some(prev) = self.before.pop() {
            self.after.push(prev);
        }
    }

    pub fn right_char(&mut self) {
        if let Some(c) = self.after.pop() {
            self.before.push(c);
        }
    }

    pub fn left_word(&mut self) {
        if self.before.len() == 0 {
            return;
        }

        let pos = match self.before[..self.before.len() - 1].rfind(|c: char| {
            c.is_whitespace() || ['.', '\\', '/', '.', ','].contains(&c)
        }) {
            Some(i) => i + 1,
            None => 0,
        };
        self.after.push_str(
            &self
                .before
                .drain(pos..self.before.len())
                .rev()
                .collect::<String>(),
        );
    }

    pub fn right_word(&mut self) {
        let pos = match self.after.trim_end().rfind(|c: char| {
            c.is_whitespace() || ['.', '\\', '/', '.', ','].contains(&c)
        }) {
            Some(p) => p,
            None => 1,
        };
        self.before
            .push_str(&self.after.drain(pos..).rev().collect::<String>());
    }

    pub fn delete_pos(&mut self) {
        self.after.pop();
    }

    pub fn clear_after(&mut self) {
        self.after = String::new();
    }

    pub fn right_all(&mut self) {
        self.before
            .push_str(&self.after.drain(..).rev().collect::<String>());
    }

    pub fn left_all(&mut self) {
        self.after
            .push_str(&self.before.drain(..).rev().collect::<String>());
    }
}

impl std::fmt::Display for Cursor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            self.before,
            self.after.chars().rev().collect::<String>()
        )
    }
}

impl std::clone::Clone for Cursor {
    fn clone(&self) -> Self {
        Cursor {
            before: self.before.clone(),
            after: self.after.clone(),
        }
    }
}

/*
 * Tests
 */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn move_cursor_right() {
        let mut initial = Cursor::from(
            "console.log".to_string(),
            r#"("hello world");"#.to_string(),
        );
        initial.right_char();
        assert_eq!(
            initial,
            Cursor::from(
                "console.log(".to_string(),
                r#""hello world");"#.to_string()
            )
        );
    }

    #[test]
    pub fn move_cursor_left() {
        let mut initial = Cursor::from(
            "console.log".to_string(),
            r#"("hello world");"#.to_string(),
        );
        initial.left_char();
        assert_eq!(
            initial,
            Cursor::from(
                "console.lo".to_string(),
                r#"g("hello world");"#.to_string()
            ),
        );
    }
}
