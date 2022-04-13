use std::collections::VecDeque;
use std::rc::Rc;

use rand::{seq::IteratorRandom, thread_rng};
use serde_cbor::ser::to_vec_packed;
use yew::{
    function_component, html, use_effect_with_deps, use_mut_ref, use_reducer, use_state, Callback,
    Html, Reducible,
};

use crate::components::hinted_word::HintedWord;
use crate::components::select_words::{SelectWords, SelectedWords};
use crate::components::turns_plot::TurnsPlot;
use crate::simulation::{SimulationInput, SimulationOutput};
use crate::word_set::{get_current_word_set, WordSet};
use crate::worker::{WordleWorkerInput, WordleWorkerOutput};
use crate::worker_atom::WordleWorkerAtom;
use crate::{EntropiesData, Hints, Word};

fn get_expected_turns(history_small: &Vec<(usize, usize)>, word_set: &WordSet) -> f64 {
    let (part_weighted_turns, prob_norm): (Vec<_>, Vec<_>) = history_small
        .iter()
        .map(|&(turns, word)| {
            let probability = word_set.dictionary.probabilities[word];
            (probability * turns as f64, probability)
        })
        .unzip();

    let (part_weighted_turns, prob_norm): (f64, f64) = (
        part_weighted_turns.into_iter().sum(),
        prob_norm.into_iter().sum(),
    );

    if prob_norm != 0. {
        part_weighted_turns / prob_norm
    } else {
        0.
    }
}

enum SimulationStateAction {
    Initialize(Word, Vec<Word>, Rc<WordSet>),
    NextStep {
        next_word: Option<Word>,
        guess: usize,
        hints: Hints,
        uncertainty: f64,
        scores: Vec<(usize, EntropiesData, f64)>,
        answers: Vec<usize>,
    },
}

#[derive(Clone, Default, PartialEq)]
struct SimulationState {
    current_turns: Vec<(f64, f64)>,
    turns_data: Vec<(f64, f64, f64)>,
    current_word: Option<Word>,
    last_hints: Option<Hints>,
    last_guess: Option<usize>,
    last_scores: Vec<(usize, f64, bool)>,
    history: VecDeque<Vec<(usize, Hints, f64)>>,
    history_small: Vec<(usize, usize)>,
    words_left: Vec<Word>,
    word_set: Option<Rc<WordSet>>,
    expected_turns: f64,
}

impl Reducible for SimulationState {
    type Action = SimulationStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            SimulationStateAction::Initialize(word, words_left, word_set) => Rc::new(Self {
                current_turns: vec![],
                turns_data: vec![],
                current_word: Some(word),
                last_hints: None,
                last_guess: None,
                last_scores: vec![],
                history: VecDeque::new(),
                history_small: vec![],
                words_left,
                word_set: Some(word_set),
                expected_turns: 0.,
            }),
            SimulationStateAction::NextStep {
                next_word,
                guess,
                hints,
                uncertainty,
                scores,
                answers,
            } => {
                let mut words_left = self.words_left.clone();
                let mut current_turns = self.current_turns.clone();
                let mut turns_data = self.turns_data.clone();
                let mut history = self.history.clone();
                let mut history_small = self.history_small.clone();
                let mut expected_turns = self.expected_turns;

                let last_scores = scores
                    .into_iter()
                    .map(|(word, _, score)| (word, score, answers.contains(&word)))
                    .collect();

                let mut last_optn: Option<&mut Vec<(usize, Hints, f64)>>;
                let history_front = if let Some(history_front) = history.front_mut() {
                    history_front
                } else {
                    history.push_front(vec![]);
                    last_optn = history.front_mut();
                    &mut **last_optn.as_mut().unwrap()
                };
                history_front.push((guess, hints.clone(), uncertainty));

                current_turns.push((uncertainty, current_turns.len() as f64));

                let word = if answers.len() == 1 {
                    let turns_num = current_turns.len();

                    let answer = answers[0];
                    if answer != guess {
                        history_front.push((answer, Hints::correct(), 0.));
                    }
                    if words_left.len() > 0 {
                        words_left.remove(0);
                    }
                    turns_data.extend(
                        current_turns
                            .iter()
                            .map(|&(uncertainty, turn)| {
                                (
                                    uncertainty,
                                    turns_num as f64 - turn - 1., // that additional one is already in the score
                                    self.word_set // as we're interested only in failed guesses
                                        .as_ref()
                                        .expect("Word set not available")
                                        .dictionary
                                        .probabilities[guess],
                                )
                            })
                            .filter(|&(_, turn, _)| turn > 0.),
                    );
                    current_turns = vec![];

                    history_small.push((history_front.len(), answer));
                    expected_turns =
                        get_expected_turns(&history_small, self.word_set.as_ref().unwrap());
                    if history.len() > 10 {
                        history.pop_back();
                    }
                    history.push_front(vec![]);
                    next_word
                } else {
                    self.current_word.clone()
                };

                Rc::new(Self {
                    current_turns,
                    turns_data,
                    current_word: word,
                    last_hints: Some(hints),
                    last_guess: Some(guess),
                    last_scores,
                    history,
                    history_small,
                    words_left: words_left,
                    word_set: self.word_set.clone(),
                    expected_turns,
                })
            }
        }
    }
}

#[function_component(Simulation)]
pub fn view() -> Html {
    let word_set = Rc::new(get_current_word_set());
    let selected_words = use_mut_ref(|| SelectedWords::Random(10));

    let simulation_state = use_reducer(|| SimulationState::default());
    let stepping = use_mut_ref(|| false);
    let next_step = use_state(|| false);
    let send_queue = use_mut_ref(|| -> Option<SimulationInput> { None });
    let words_left = use_mut_ref(|| -> Vec<Word> { vec![] });
    *words_left.borrow_mut() = simulation_state.words_left.clone();

    let on_words_set = {
        let selected_words = selected_words.clone();
        Callback::from(move |new_selected_words| {
            *selected_words.borrow_mut() = new_selected_words;
        })
    };

    let cb = {
        let send_queue = send_queue.clone();
        let simulation_state = simulation_state.clone();
        let words_left = words_left.clone();
        move |output: WordleWorkerOutput| match output {
            WordleWorkerOutput::SetWordSet(name) => log::info!("Set worker with: {name}"),
            WordleWorkerOutput::Simulation(output) => match output {
                SimulationOutput::StepComplete {
                    guess,
                    hints,
                    uncertainty,
                    scores,
                    answers,
                } => {
                    let next_guess = scores
                        .iter()
                        .filter(|&&(candidate, ..)| candidate != guess)
                        .next()
                        .unwrap()
                        .0;

                    let next_word = if answers.len() > 1 {
                        match send_queue.try_borrow_mut() {
                            Ok(ref mut send_queue) => {
                                **send_queue = Some(SimulationInput::Continue(Some(next_guess)));
                            }
                            _ => log::info!("Unable to borrow in worker callback 1"),
                        }
                        None
                    } else {
                        if words_left.borrow().len() > 0 {
                            let next_word = &words_left.borrow()[0];
                            log::info!("Starting next word: {next_word}");
                            match send_queue.try_borrow_mut() {
                                Ok(ref mut send_queue) => {
                                    **send_queue =
                                        Some(SimulationInput::Start(next_word.clone(), None));
                                }
                                _ => log::info!("Unable to borrow in worker callback 2"),
                            }
                            Some(next_word.clone())
                        } else {
                            None
                        }
                    };

                    simulation_state.dispatch(SimulationStateAction::NextStep {
                        next_word,
                        guess,
                        hints,
                        uncertainty,
                        scores,
                        answers,
                    })
                }
                SimulationOutput::Stopped => todo!(),
            },
            WordleWorkerOutput::Err(error) => {
                log::info!("{error}");
            }
            _ => log::info!("Unexpected worker output"),
        }
    };

    let worker = WordleWorkerAtom::with_callback(Rc::new(cb));

    if !*stepping.borrow() || *next_step {
        match send_queue.try_borrow_mut() {
            Ok(ref mut send_queue) => {
                if let Some(input) = send_queue.take() {
                    worker.send(WordleWorkerInput::Simulation(input));
                    next_step.set(false);
                }
            }
            _ => log::info!("unable to borrow in queue resolution"),
        }
    }

    {
        let worker = worker.clone();
        let word_set = word_set.clone();
        let word_set_name = word_set.name.clone();
        use_effect_with_deps(
            move |_| {
                worker.send(WordleWorkerInput::SetWordSetEncoded(
                    to_vec_packed(&word_set.reduce_entropies(10)).unwrap(),
                ));
                || ()
            },
            word_set_name,
        )
    }

    let on_start_button_click = {
        let stepping = stepping.clone();
        let worker = worker.clone();
        let word_set = word_set.clone();
        let selected_words = selected_words.clone();
        let simulation_state = simulation_state.clone();
        let word_set = word_set.clone();

        Callback::from(move |_| {
            *stepping.borrow_mut() = false;
            let mut words = match *selected_words.borrow() {
                SelectedWords::Random(n) => {
                    let mut rng = thread_rng();
                    word_set
                        .dictionary
                        .words
                        .iter()
                        .cloned()
                        .choose_multiple(&mut rng, n)
                }
                SelectedWords::Custom(ref words) => words.clone(),
            };

            if words.len() > 0 {
                let word = words.remove(0);
                simulation_state.dispatch(SimulationStateAction::Initialize(
                    word.clone(),
                    words,
                    word_set.clone(),
                ));
                worker.send(WordleWorkerInput::Simulation(SimulationInput::Start(
                    word, None,
                )));
            } else {
                log::info!("..no words?");
            }
        })
    };

    let on_step_button_click = {
        let stepping = stepping.clone();
        let next_step = next_step.clone();
        let worker = worker.clone();

        Callback::from(move |_| {
            *stepping.borrow_mut() = true;

            let input = send_queue.borrow_mut().take();
            if let Some(input) = input {
                worker.send(WordleWorkerInput::Simulation(input));
            } else {
                next_step.set(true);
            }
        })
    };

    let on_continue_button_click = {
        let stepping = stepping.clone();
        let next_step = next_step.clone();

        Callback::from(move |_| {
            *stepping.borrow_mut() = false;
            next_step.set(false);
        })
    };

    let data = simulation_state.turns_data.clone();
    let words_len = word_set.dictionary.words.len();
    let running = !simulation_state.words_left.is_empty();

    log::info!("{:#?}", simulation_state.history);

    html! {
        <section>
            <SelectWords dictionary={word_set.dictionary.clone()} {on_words_set} />
            <button class="btn btn-primary" onclick={on_start_button_click}>{ "Start" }</button>
            <button
                class="btn btn-primary"
                onclick={on_continue_button_click}
                disabled={!*stepping.borrow() || !running}
            > { "Continue" }</button>
            <button
                class="btn btn-primary"
                onclick={on_step_button_click}
                disabled={!running}
            >{ "Step" }</button>
            <div class="columns">
                <div class="column col-8 col-xl-12">
                    <TurnsPlot {data} {words_len} title={format!("Expected turns: {}", simulation_state.expected_turns)} />
                </div>
                <div class="column col-3 col-xl-9">
                    <ul class="words_left_list">
                        {
                            simulation_state.last_scores.iter().map(|&(word, score, could_be_answer)| {
                                let word = &word_set.dictionary.words[word];
                                html! {
                                    <li> { format!("{word}: {score} | {could_be_answer:#?}")} </li>
                                }
                            }).collect::<Html>()
                        }
                    </ul>
                </div>
                <div class="column col-1 col-xl-3">
                    <div>
                        if let Some(ref word) = simulation_state.current_word {
                            <p> { word } </p>
                        }
                        <p> { "Left:" } </p>
                        <ul class="words_left_list">
                            {
                                simulation_state.words_left.iter().map(|word| {
                                    html! {
                                        <li> { word } </li>
                                    }
                                }).collect::<Html>()
                            }
                        </ul>
                    </div>
                </div>
            </div>
            <div>
                {
                    simulation_state.history.iter().map(|row| {
                        html! {
                            <p>
                                {
                                    row.iter().map(|(word, hints, _)| {
                                        let word = word_set.dictionary.words[*word].clone();
                                        let hints = hints.clone();
                                        html! {
                                            <>
                                                <HintedWord {word} {hints} />
                                                <div style="display: inline" class="m-2" />
                                                <div style="display: inline" class="m-2" />
                                            </>
                                        }
                                    }).collect::<Html>()
                                }
                            </p>
                        }
                    }).collect::<Html>()
                }
            </div>
        </section>
    }
}
