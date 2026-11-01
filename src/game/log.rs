//! The message log.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Info,
    Good,
    Bad,
    Combat,
    Loot,
    Special,
}

#[derive(Clone, Debug)]
pub struct Message {
    pub text: String,
    pub tone: Tone,
    pub repeats: u32,
}

impl Message {
    pub fn display(&self) -> String {
        if self.repeats > 1 { format!("{} (x{})", self.text, self.repeats) } else { self.text.clone() }
    }
}

const CAPACITY: usize = 200;

#[derive(Clone, Debug, Default)]
pub struct Log {
    messages: Vec<Message>,
}

impl Log {
    pub fn push(&mut self, text: impl Into<String>, tone: Tone) {
        let text = text.into();
        if let Some(last) = self.messages.last_mut()
            && last.text == text
        {
            last.repeats += 1;
            return;
        }
        self.messages.push(Message { text, tone, repeats: 1 });
        if self.messages.len() > CAPACITY {
            self.messages.remove(0);
        }
    }

    pub fn info(&mut self, text: impl Into<String>) {
        self.push(text, Tone::Info);
    }

    /// The most recent `n` messages, oldest first.
    pub fn recent(&self, n: usize) -> &[Message] {
        let start = self.messages.len().saturating_sub(n);
        &self.messages[start..]
    }
}
