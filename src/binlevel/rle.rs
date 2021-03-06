use std::iter::Peekable;

pub fn make_runs<I: Iterator<Item = u8>>(input: I) -> RunIter<I> {
    runs(input)
}

#[derive(Debug, Clone, Copy)]
pub struct Run {
    pub val: u8,
    pub length: usize,
}

pub struct RunIter<I: Iterator<Item = u8>> {
    it: Peekable<I>,
}

impl<I: Iterator<Item = u8>> Iterator for RunIter<I> {
    type Item = Run;
    fn next(&mut self) -> Option<Self::Item> {

        // We use peeking to avoid eating the start of the next run.

        // if we hit the end of the original data, there’s no more runs
        if self.it.peek().is_none() {
            return None;
        };

        // unwrap() is safe; if we got here, there must be a next item
        let mut run = Run {
            val: self.it.next().unwrap(),
            length: 1,
        };
        // a run can continue until the end of the iteration
        while let Some(&v) = self.it.peek() {
            // reached the end of the run because we hit a different value
            if v != run.val {
                break;
            }
            run.length += 1;
            // advance
            self.it.next();
        }

        Some(run)
    }
}

fn runs<'a, I: Iterator<Item = u8>>(xs: I) -> RunIter<I> {
    RunIter { it: xs.peekable() }
}
