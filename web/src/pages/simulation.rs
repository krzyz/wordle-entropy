use std::rc::Rc;

use rand::{seq::IteratorRandom, thread_rng};
use serde_cbor::ser::to_vec_packed;
use yew::{
    function_component, html, use_effect_with_deps, use_mut_ref, use_reducer, Callback, Html,
    Reducible,
};

use crate::components::hinted_word::HintedWord;
use crate::components::select_words::{SelectWords, SelectedWords};
use crate::components::turns_plot::TurnsPlot;
use crate::simulation::{SimulationInput, SimulationOutput};
use crate::word_set::{get_current_word_set, WordSet};
use crate::worker::{WordleWorkerInput, WordleWorkerOutput};
use crate::worker_atom::WordleWorkerAtom;
use crate::{EntropiesData, Hints, Word};

enum SimulationStateAction {
    Initialize(Word, Vec<Word>, Rc<WordSet>),
    NextStep {
        next_word: Option<Word>,
        guess: Word,
        hints: Hints,
        uncertainty: f64,
        scores: Vec<(EntropiesData, f64)>,
        answers: Vec<usize>,
    },
}

#[derive(Clone, Default, PartialEq)]
struct SimulationState {
    current_turns: Vec<(f64, f64)>,
    turns_data: Vec<(f64, f64)>,
    current_word: Option<Word>,
    last_hints: Option<Hints>,
    last_guess: Option<Word>,
    history: Vec<Vec<(Word, Hints, f64)>>,
    words_left: Vec<Word>,
    word_set: Option<Rc<WordSet>>,
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
                history: vec![],
                words_left,
                word_set: Some(word_set),
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

                let mut last_optn: Option<&mut Vec<(Word, Hints, f64)>>;
                let history_last = if let Some(history_last) = history.last_mut() {
                    history_last
                } else {
                    history.push(vec![]);
                    last_optn = history.last_mut();
                    &mut **last_optn.as_mut().unwrap()
                };
                history_last.push((guess.clone(), hints.clone(), uncertainty));

                let word = if answers.len() == 1 {
                    let mut turns_num = current_turns.len();

                    let correct_answer =
                        &self.word_set.as_ref().unwrap().dictionary.words[answers[0]];
                    if correct_answer != &guess {
                        history_last.push((correct_answer.clone(), Hints::correct(), 0.));
                        turns_num += 1;
                    }
                    if words_left.len() > 0 {
                        words_left.remove(0);
                    }
                    turns_data.extend(
                        current_turns
                            .iter()
                            .map(|&(uncertainty, turn)| (uncertainty, turns_num as f64 - turn)),
                    );
                    current_turns = vec![];
                    history.push(vec![]);
                    next_word
                } else {
                    current_turns.push((uncertainty, current_turns.len() as f64));
                    self.current_word.clone()
                };

                Rc::new(Self {
                    current_turns,
                    turns_data,
                    current_word: word,
                    last_hints: Some(hints),
                    last_guess: Some(guess),
                    history,
                    words_left: words_left,
                    word_set: self.word_set.clone(),
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
                    let words = scores
                        .iter()
                        .take(10)
                        .map(|(EntropiesData { word, entropy, .. }, _)| {
                            format!("{word}: {entropy}")
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    log::info!(
                        "{guess}, {hints}, {uncertainty}, {words}, {:#?}",
                        answers.iter().take(10).collect::<Vec<_>>()
                    );

                    let next_guess = scores.iter().next().unwrap().0.word.clone();

                    let next_word = if answers.len() > 1 {
                        *send_queue.borrow_mut() =
                            Some(SimulationInput::Continue(Some(next_guess)));
                        None
                    } else {
                        if words_left.borrow().len() > 0 {
                            let next_word = &words_left.borrow()[0];
                            log::info!("Starting next word: {next_word}");
                            *send_queue.borrow_mut() =
                                Some(SimulationInput::Start(next_word.clone(), None));
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

    if let Some(input) = send_queue.borrow_mut().take() {
        let worker = worker.clone();
        worker.send(WordleWorkerInput::Simulation(input));
    }

    {
        let worker = worker.clone();
        let word_set = word_set.clone();
        let word_set_name = word_set.name.clone();
        use_effect_with_deps(
            move |_| {
                log::info!("send set work set");
                worker.send(WordleWorkerInput::SetWordSetEncoded(
                    to_vec_packed(&word_set.reduce_entropies(10)).unwrap(),
                ));
                log::info!("finished send");
                || ()
            },
            word_set_name,
        )
    }

    let on_run_button_click = {
        let worker = worker.clone();
        let word_set = word_set.clone();
        let selected_words = selected_words.clone();
        let simulation_state = simulation_state.clone();
        let word_set = word_set.clone();

        Callback::from(move |_| {
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

    let data = simulation_state.turns_data.clone();
    let words_len = word_set.dictionary.words.len();

    html! {
        <section>
            <SelectWords dictionary={word_set.dictionary.clone()} {on_words_set} />
            <button class="btn btn-primary" onclick={on_run_button_click}>{ "Run" }</button>
            <div class="columns">
                <div class="column">
                    <TurnsPlot {data} {words_len} />
                </div>
                <div class="column">
                    <div>
                        if let Some(ref word) = simulation_state.current_word {
                            <p> { word } </p>
                        }
                        if let (Some(word), Some(hints)) = (simulation_state.last_guess.clone(), simulation_state.last_hints.clone()) {
                            <p> <HintedWord {word} {hints} /></p>
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
                                    row.iter().map(|(word, hints, _uncertainty)| {
                                        let word = word.clone();
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
