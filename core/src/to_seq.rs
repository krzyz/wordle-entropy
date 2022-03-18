use core::option::Option;
use std::iter;

use crossbeam::channel;
use crossbeam::channel::{Receiver, Sender};
use rayon::iter::ParallelIterator;

pub trait IntoSequentialIteratorEx<'a, T: Sized>: Sized {
    fn into_seq_iter(self) -> Box<dyn 'a + Iterator<Item=T>>;
}

impl<'a, T, PI> IntoSequentialIteratorEx<'a, T> for PI
    where
        T: 'a + Send,
        PI: 'a + ParallelIterator<Item=T>,
{
    fn into_seq_iter(self) -> Box<dyn 'a + Iterator<Item=T>> {
        let (sender, receiver) = channel::unbounded();

        Box::new(deferred_first_element(self, sender, receiver.clone())
            .chain(deferred_remaining_elements(receiver)))
    }
}

fn deferred_first_element<'a, T: 'a + Send, PI: 'a + ParallelIterator<Item=T>>(
    par_iter: PI,
    sender: Sender<T>,
    receiver: Receiver<T>) -> Box<dyn 'a + Iterator<Item=T>>
{
    let deferred = iter::once(Box::new(move || {
        crossbeam::scope(|s| {
            s.spawn(|_| {
                par_iter.for_each(|element| {
                    sender.send(element).unwrap();
                });

                drop(sender);
            });
        }).unwrap();

        receiver.recv().ok()
    }) as Box<dyn FnOnce() -> Option<T>>);

    Box::new(deferred
        .map(|f| {
            f()
        })
        .filter(Option::is_some)
        .map(Option::unwrap))
}

fn deferred_remaining_elements<'a, T: 'a + Send>(receiver: Receiver<T>) -> Box<dyn 'a + Iterator<Item=T>> {
    Box::new(
        iter::repeat_with(move || {
            receiver.recv().ok()
        })
            .take_while(Option::is_some)
            .map(Option::unwrap))
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use rayon::iter::{IntoParallelIterator, ParallelBridge};

    use super::*;

    #[test]
    fn does_noting_for_an_empty_iterator() {
        // when
        let result = Vec::<i32>::new().into_par_iter().into_seq_iter().collect_vec();

        // then
        assert_eq!(Vec::<i32>::new(), result);
    }

    #[test]
    fn iterates_over_whole_iterator_range() {
        // given
        let elements = 120;

        // when
        let result = iter::repeat(12)
            .take(elements)
            .par_bridge()
            .into_seq_iter()
            .count();

        // then
        assert_eq!(elements, result);
    }
}