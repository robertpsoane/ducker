#[derive(Clone)]
pub enum Message<I, T> {
    Tick,
    Input(I),
    Transition(T),
}

#[derive(PartialEq, Debug, Clone)]
pub enum MessageResponse {
    Consumed,
    NotConsumed,
}

impl MessageResponse {
    pub fn is_consumed(&self) -> bool {
        *self == Self::Consumed
    }
}

impl From<bool> for MessageResponse {
    fn from(consumed: bool) -> Self {
        if consumed {
            Self::Consumed
        } else {
            Self::NotConsumed
        }
    }
}
