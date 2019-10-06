use std::ops;
use std::fmt;
use std::cmp::{self, Ordering};
use std::collections::BinaryHeap;

#[derive(Ord, PartialOrd, PartialEq, Eq, Copy, Clone, Debug)]
pub struct Time {
    pub ticks: u32,
    pub micro_ticks: u32
}

impl Default for Time {
    fn default() -> Self {
        Time::new(0, 0)
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let precision = cmp::max(0, cmp::min(6, f.precision().unwrap_or(6)));
        if precision == 0 {
            write!(f, "{}", self.ticks)
        } else {
            let mut d = 6 - precision;
            let mut micro_ticks = self.micro_ticks;
            while d > 0 {
                micro_ticks /= 10;
                d -= 1;
            }
            write!(f, "{}.{:0width$}", self.ticks, micro_ticks, width=precision)
        }
    }
}

impl Time {
    pub fn new(ticks: u32, micro_ticks: u32) -> Self {
        let mut time = Time { ticks, micro_ticks };
        time.normalise();
        time
    }

    pub fn normalise(&mut self) {
        self.ticks += self.micro_ticks / 1000000;
        self.micro_ticks = self.micro_ticks % 1000000;
    }
}

impl ops::Add for Time {
    type Output = Time;

    fn add(self, rhs: Time) -> Self::Output {
        Time::new(self.ticks + rhs.ticks, self.micro_ticks + rhs.micro_ticks)
    }
}

impl ops::AddAssign for Time {
    fn add_assign(&mut self, rhs: Time) {
        self.ticks += rhs.ticks;
        self.micro_ticks += rhs.micro_ticks;
        self.normalise();
    }
}

impl ops::Add<u32> for Time {
    type Output = Time;

    fn add(self, rhs: u32) -> Self::Output {
        Time::new(self.ticks + rhs, self.micro_ticks)
    }
}

impl ops::AddAssign<u32> for Time {
    fn add_assign(&mut self, rhs: u32) {
        self.ticks += rhs;
    }
}

struct Event<T> {
    pub time: Time,
    pub generation: u32,
    pub item: T,
}

impl <T> PartialEq for Event<T> {
    fn eq(&self, other: &Event<T>) -> bool {
        self.time == other.time && self.generation == other.generation
    }
}

impl <T> Eq for Event<T> {

}

impl <T> Ord for Event<T> {
    fn cmp(&self, other: &Event<T>) -> Ordering {
        other.time.cmp(&self.time)
            .then_with(|| other.generation.cmp(&self.generation))
    }
}

impl <T> PartialOrd for Event<T> {
    fn partial_cmp(&self, other: &Event<T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct EventQueue<T> {
    generation: u32,
    queue: BinaryHeap<Event<T>>,
}

impl <T> Default for EventQueue<T> {
    fn default() -> Self {
        EventQueue::new()
    }
}

impl <T> EventQueue<T> {
    pub fn new() -> Self {
        EventQueue {
            generation: 0,
            queue: BinaryHeap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn add(&mut self, when: Time, item: T) {
        self.generation += 1;
        self.queue.push(Event { time: when, generation: self.generation, item: item })
    }

    pub fn next(&mut self) -> Option<(Time, T)> {
        self.queue
            .pop()
            .map(|event| (event.time, event.item))
    }

    pub fn peek<'a>(&'a self) -> Option<(&'a Time, &'a T)> {
        self.queue
            .peek()
            .map(|event| (&event.time, &event.item))
    }

    pub fn has_next(&self) -> bool {
        !self.queue.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_queue() {
        let mut q = EventQueue::new();
        q.add(Time::default() + 1, "b");
        q.add(Time::default(), "a");
        q.add(Time::default(), "a2");

        assert!(q.has_next());
        assert_eq!(q.len(), 3);
        assert_eq!(q.peek(), Some((&Time::default(), &"a")));
        assert_eq!(q.next(), Some((Time::default(), "a")));
        assert_eq!(q.next(), Some((Time::default(), "a2")));
        assert_eq!(q.next(), Some((Time::default() + 1, "b")));
        assert_eq!(q.next(), None);
        assert_eq!(q.has_next(), false);
    }

    #[test]
    pub fn test_display_time() {
        let t = Time::new(1, 1000);
        
        assert_eq!(format!("{}", t), "1.001000");
        assert_eq!(format!("{:.0}", t), "1");
        assert_eq!(format!("{:.3}", t), "1.001");
        assert_eq!(format!("{:.6}", t), "1.001000");
    }
}