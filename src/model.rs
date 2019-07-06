use crate::configs::Config;
use termion::event::Key;

pub enum UiWidget {
    Student,
    Status,
    Log,
    Diff,
    Config,
}

pub struct Model {
    pub config: Config,
    pub current: UiWidget,
}

impl Model {
    pub fn new(config: Config) -> Model {
        Model {
            config,
            current: UiWidget::Student,
        }
    }

    pub fn handle(&mut self, key: Key) {
        match key {
            // change current widget
            Key::Char('H') => {
                self.current = match self.current {
                    UiWidget::Student => UiWidget::Student,
                    UiWidget::Status => UiWidget::Status,
                    UiWidget::Log => UiWidget::Student,
                    UiWidget::Diff => UiWidget::Student,
                    UiWidget::Config => UiWidget::Status,
                };
            }
            Key::Char('J') => {
                self.current = match self.current {
                    UiWidget::Student => UiWidget::Status,
                    UiWidget::Status => UiWidget::Status,
                    UiWidget::Log => UiWidget::Diff,
                    UiWidget::Diff => UiWidget::Config,
                    UiWidget::Config => UiWidget::Config,
                };
            }
            Key::Char('K') => {
                self.current = match self.current {
                    UiWidget::Student => UiWidget::Student,
                    UiWidget::Status => UiWidget::Student,
                    UiWidget::Log => UiWidget::Log,
                    UiWidget::Diff => UiWidget::Log,
                    UiWidget::Config => UiWidget::Diff,
                };
            }
            Key::Char('L') => {
                self.current = match self.current {
                    UiWidget::Student => UiWidget::Log,
                    UiWidget::Status => UiWidget::Config,
                    UiWidget::Log => UiWidget::Log,
                    UiWidget::Diff => UiWidget::Diff,
                    UiWidget::Config => UiWidget::Config,
                };
            }
            _ => {}
        }
    }
}
