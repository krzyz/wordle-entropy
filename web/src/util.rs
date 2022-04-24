use crate::EntropiesData;

pub fn scores_without_full_data(
    scores: Vec<(usize, EntropiesData, f64)>,
) -> Vec<(usize, f64, f64)> {
    scores
        .into_iter()
        .map(|(word, entropies_data, left_turns)| (word, entropies_data.entropy, left_turns))
        .collect::<Vec<_>>()
}
