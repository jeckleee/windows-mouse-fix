use crate::model::{Action, Config, Gesture, Modifiers, Trigger};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    Action(Action),
    Replay {
        button: u8,
        modifiers: Modifiers,
        count: u8,
    },
    Captured(Trigger),
}

struct Cycle {
    modifiers: Modifiers,
    down: bool,
    at: u64,
    clicks: u8,
    consumed: bool,
    capture: bool,
    dx: i32,
    dy: i32,
}

pub struct Engine {
    pub config: Config,
    pub capture: bool,
    pub double_ms: u64,
    cycles: [Option<Cycle>; 3],
    effects: Vec<Effect>,
}

impl Engine {
    pub fn new(config: Config, double_ms: u64) -> Self {
        Self {
            config,
            capture: false,
            double_ms,
            cycles: [None, None, None],
            effects: vec![],
        }
    }

    pub fn configure(&mut self, config: Config) {
        self.cancel();
        self.config = config;
    }

    // Keep consumed downs until their corresponding physical ups, even after pause/edit.
    pub fn cancel(&mut self) {
        for slot in &mut self.cycles {
            if let Some(cycle) = slot {
                if cycle.down {
                    cycle.consumed = true;
                    cycle.capture = true;
                } else {
                    *slot = None;
                }
            }
        }
        self.capture = false;
        self.effects.clear();
    }

    pub fn has_active_input(&self) -> bool {
        self.cycles.iter().any(Option::is_some)
    }

    fn has(&self, button: u8, modifiers: Modifiers, gesture: Gesture) -> bool {
        self.config.rules.iter().any(|r| {
            r.trigger
                == Trigger {
                    button,
                    modifiers,
                    gesture,
                }
        })
    }

    fn emit(&mut self, button: u8, cycle: &Cycle, gesture: Gesture) {
        let trigger = Trigger {
            button,
            modifiers: cycle.modifiers,
            gesture,
        };
        if cycle.capture {
            self.effects.push(Effect::Captured(trigger));
        } else if let Some(rule) = self.config.rules.iter().find(|r| r.trigger == trigger) {
            self.effects.push(Effect::Action(rule.action.clone()));
        } else {
            self.effects.push(Effect::Replay {
                button,
                modifiers: cycle.modifiers,
                count: cycle.clicks.max(1),
            });
        }
    }

    pub fn button(&mut self, button: u8, down: bool, modifiers: Modifiers, now: u64) -> bool {
        if !(3..=5).contains(&button) {
            return false;
        }
        self.tick(now);
        let index = (button - 3) as usize;
        if down {
            let old = self.cycles[index].take();
            let mut clicks = 0;
            if let Some(cycle) = old {
                if cycle.down {
                    self.cycles[index] = Some(cycle);
                    return true;
                }
                if cycle.modifiers == modifiers && cycle.capture == self.capture {
                    clicks = cycle.clicks;
                } else {
                    self.emit(button, &cycle, Gesture::Click);
                }
            }
            let mapped = self.config.enabled
                && self
                    .config
                    .rules
                    .iter()
                    .any(|r| r.trigger.button == button && r.trigger.modifiers == modifiers);
            if !self.capture && !mapped {
                return false;
            }
            let mut cycle = Cycle {
                modifiers,
                down: true,
                at: now,
                clicks,
                consumed: false,
                capture: self.capture,
                dx: 0,
                dy: 0,
            };
            if !cycle.capture
                && self.has(button, modifiers, Gesture::Click)
                && !Gesture::ALL
                    .into_iter()
                    .any(|g| g != Gesture::Click && self.has(button, modifiers, g))
            {
                self.emit(button, &cycle, Gesture::Click);
                cycle.consumed = true;
            }
            self.cycles[index] = Some(cycle);
            true
        } else if let Some(mut cycle) = self.cycles[index].take() {
            if !cycle.down {
                self.cycles[index] = Some(cycle);
                return false;
            }
            if cycle.consumed {
                return true;
            }
            cycle.down = false;
            cycle.clicks += 1;
            cycle.at = now;
            if cycle.clicks >= 2 {
                self.emit(button, &cycle, Gesture::DoubleClick);
            } else if cycle.capture || self.has(button, cycle.modifiers, Gesture::DoubleClick) {
                self.cycles[index] = Some(cycle);
            } else {
                self.emit(button, &cycle, Gesture::Click);
            }
            true
        } else {
            false
        }
    }

    pub fn freeze_pointer(&self) -> bool {
        self.config.enabled
            && self.config.lock_pointer
            && self.cycles.iter().enumerate().any(|(i, slot)| {
                slot.as_ref().is_some_and(|c| {
                    c.down
                        && !c.capture
                        && Gesture::ALL
                            .into_iter()
                            .any(|g| g.is_drag() && self.has(i as u8 + 3, c.modifiers, g))
                })
            })
    }

    pub fn movement(&mut self, dx: i32, dy: i32) {
        for index in 0..3 {
            let Some(mut cycle) = self.cycles[index].take() else {
                continue;
            };
            if cycle.down && !cycle.consumed {
                cycle.dx = cycle.dx.saturating_add(dx);
                cycle.dy = cycle.dy.saturating_add(dy);
                if cycle.dx.unsigned_abs().max(cycle.dy.unsigned_abs()) >= 24 {
                    let gesture = if cycle.dx.unsigned_abs() > cycle.dy.unsigned_abs() {
                        if cycle.dx < 0 {
                            Gesture::DragLeft
                        } else {
                            Gesture::DragRight
                        }
                    } else if cycle.dy < 0 {
                        Gesture::DragUp
                    } else {
                        Gesture::DragDown
                    };
                    if cycle.capture || self.has(index as u8 + 3, cycle.modifiers, gesture) {
                        self.emit(index as u8 + 3, &cycle, gesture);
                        cycle.consumed = true;
                    }
                }
            }
            self.cycles[index] = Some(cycle);
        }
    }

    pub fn tick(&mut self, now: u64) {
        for index in 0..3 {
            let Some(mut cycle) = self.cycles[index].take() else {
                continue;
            };
            let button = index as u8 + 3;
            if !cycle.consumed {
                if cycle.down
                    && now.saturating_sub(cycle.at) >= 400
                    && (cycle.capture || self.has(button, cycle.modifiers, Gesture::Hold))
                {
                    self.emit(button, &cycle, Gesture::Hold);
                    cycle.consumed = true;
                } else if !cycle.down && now.saturating_sub(cycle.at) >= self.double_ms {
                    self.emit(button, &cycle, Gesture::Click);
                    continue;
                }
            }
            self.cycles[index] = Some(cycle);
        }
    }

    pub fn drain(&mut self) -> Vec<Effect> {
        std::mem::take(&mut self.effects)
    }
}
