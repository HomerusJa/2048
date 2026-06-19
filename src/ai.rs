use crate::board::{Board, Direction};

pub trait Evaluator {
    fn evaluate(&self, board: &Board) -> f32;
}

// TODO: Implement heuristics

pub struct Bot<E: Evaluator> {
    max_depth: u8,
    evaluator: E,
}

impl<E: Evaluator> Bot<E> {
    pub fn new(max_depth: u8, evaluator: E) -> Self {
        Self {
            max_depth,
            evaluator,
        }
    }

    pub fn get_best_move(&self, board: Board) -> Option<Direction> {
        todo!("To be implemented!")
    }
}
