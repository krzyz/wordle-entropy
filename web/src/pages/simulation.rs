use std::collections::VecDeque;
use std::rc::Rc;
use std::str::FromStr;

use bounce::use_atom_setter;
use rand::{seq::IteratorRandom, seq::SliceRandom, thread_rng};
use serde_cbor::ser::to_vec_packed;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use web_sys::HtmlElement;
use yew::{
    classes, function_component, html, use_effect_with_deps, use_mut_ref, use_reducer, use_state,
    use_state_eq, Callback, Html, MouseEvent, Reducible, TargetCast,
};

use crate::components::{
    Calibration, SelectWords, SelectedWords, SimulationDetail, SimulationHistory, ToastOption,
    ToastType,
};
use crate::simulation::{SimulationInput, SimulationOutput};
use crate::util::scores_without_full_data;
use crate::word_set::{get_current_word_set, WordSet};
use crate::worker::{WordleWorkerInput, WordleWorkerOutput};
use crate::worker_atom::WordleWorkerAtom;
use crate::{EntropiesData, Hints};

#[derive(Clone, Debug, PartialEq)]
pub struct GuessStep {
    // word, entropy, left_turns
    pub guess: usize,
    pub hints: usize,
    pub uncertainty: f64,
    pub scores: Vec<(usize, f64, f64)>,
    pub answers: Vec<usize>,
}

enum SimulationStateAction {
    Reset,
    Initialize(usize, Vec<usize>, Rc<WordSet>),
    NextStep {
        next_word: Option<usize>,
        guess: usize,
        hints: usize,
        uncertainty: f64,
        scores: Vec<(usize, EntropiesData, f64)>,
        answers: Vec<usize>,
    },
}

#[derive(Clone, Default, PartialEq)]
struct SimulationState {
    current_turns: Vec<(f64, f64)>,
    turns_data: Vec<(f64, f64, f64)>,
    current_word: Option<usize>,
    history: VecDeque<(usize, Vec<GuessStep>)>,
    history_small: Vec<(usize, usize)>,
    words_left: Vec<usize>,
    word_set: Option<Rc<WordSet>>,
}

impl Reducible for SimulationState {
    type Action = SimulationStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            SimulationStateAction::Reset => Rc::new(Self::default()),
            SimulationStateAction::Initialize(word, words_left, word_set) => Rc::new(Self {
                current_turns: vec![],
                turns_data: vec![],
                current_word: Some(word),
                history: VecDeque::new(),
                history_small: vec![],
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
                let mut history_small = self.history_small.clone();

                if let None = self.current_word {
                    return self.clone();
                }

                let mut last_optn: Option<&mut (usize, Vec<GuessStep>)>;
                let history_front = if let Some(history_front) = history.front_mut() {
                    history_front
                } else {
                    history.push_front((self.current_word.unwrap_or(0), vec![]));
                    last_optn = history.front_mut();
                    &mut **last_optn.as_mut().unwrap()
                };
                let scores = scores_without_full_data(scores);

                history_front.1.push(GuessStep {
                    guess,
                    hints,
                    uncertainty,
                    scores: scores.clone(),
                    answers: answers.clone(),
                });

                current_turns.push((uncertainty, current_turns.len() as f64));

                let word = if answers.len() == 1 {
                    let turns_num = current_turns.len();

                    let answer = answers[0];
                    if answer != guess {
                        history_front.1.push(GuessStep {
                            guess: answer,
                            hints: Hints::correct().to_ind(),
                            uncertainty: 0.,
                            scores,
                            answers: vec![answer],
                        });
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

                    history_small.push((history_front.1.len(), answer));
                    if history.len() > 10 {
                        history.pop_back();
                    }
                    if let Some(current_word) = self.current_word {
                        history.push_front((current_word, vec![]));
                    }
                    next_word
                } else {
                    self.current_word.clone()
                };

                Rc::new(Self {
                    current_turns,
                    turns_data,
                    current_word: word,
                    history,
                    history_small,
                    words_left: words_left,
                    word_set: self.word_set.clone(),
                })
            }
        }
    }
}

#[derive(PartialEq, EnumIter, Display, EnumString)]
pub enum Tab {
    History,
    Detail,
    Calibration,
}

#[function_component(Simulation)]
pub fn view() -> Html {
    let word_set = Rc::new(get_current_word_set());
    let selected_words = use_mut_ref(|| SelectedWords::Random(10));

    let simulation_state = use_reducer(|| SimulationState::default());
    let stepping = use_mut_ref(|| false);
    let next_step = use_state(|| false);
    let send_queue = use_mut_ref(|| -> Option<SimulationInput> { None });
    let words_left = use_mut_ref(|| -> Vec<usize> { vec![] });
    let set_toast = use_atom_setter::<ToastOption>();
    let active_tab = use_state_eq(|| Tab::History);
    let all_words = use_mut_ref(|| -> Vec<usize> { vec![] });

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
        let set_toast = set_toast.clone();

        move |output: WordleWorkerOutput| match output {
            WordleWorkerOutput::SetWordSet(_name) => (),
            WordleWorkerOutput::Simulation(output) => match output {
                SimulationOutput::StepComplete { hints: None, .. } => {
                    set_toast(ToastOption::new(
                        "Expected hints to be available from the simulation step".to_string(),
                        ToastType::Error,
                    ));
                }
                SimulationOutput::StepComplete {
                    guess,
                    hints: Some(hints),
                    uncertainty,
                    scores,
                    answers,
                    ..
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
                                **send_queue = Some(SimulationInput::Continue {
                                    hints: None,
                                    guess: Some(next_guess),
                                });
                            }
                            _ => log::error!("Unable to borrow in worker callback 1"),
                        }
                        None
                    } else {
                        if words_left.borrow().len() > 0 {
                            let next_word = &words_left.borrow()[0];
                            match send_queue.try_borrow_mut() {
                                Ok(ref mut send_queue) => {
                                    **send_queue = Some(SimulationInput::StartKnownAnswer {
                                        correct: next_word.clone(),
                                        guess: None,
                                    });
                                }
                                _ => log::error!("Unable to borrow in worker callback 2"),
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
            WordleWorkerOutput::Err(err) => {
                simulation_state.dispatch(SimulationStateAction::Reset);
                set_toast(ToastOption::new(
                    format!("Worker error: {err}").to_string(),
                    ToastType::Error,
                ))
            }
            _ => {
                simulation_state.dispatch(SimulationStateAction::Reset);
                set_toast(ToastOption::new(
                    "Unexpected worker output".to_string(),
                    ToastType::Error,
                ))
            }
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
            _ => log::error!("unable to borrow in queue resolution"),
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
        let all_words = all_words.clone();
        let set_toast = set_toast.clone();

        Callback::from(move |_| {
            *stepping.borrow_mut() = false;
            let mut words = match *selected_words.borrow() {
                SelectedWords::Random(n) => {
                    let mut rng = thread_rng();
                    let mut words_selected =
                        (0..word_set.dictionary.words.len()).choose_multiple(&mut rng, n);
                    words_selected.shuffle(&mut rng);
                    words_selected
                }
                SelectedWords::Custom(ref words) => words.clone(),
            };

            *all_words.borrow_mut() = words.clone();

            if words.len() > 0 {
                let word = words.remove(0);
                simulation_state.dispatch(SimulationStateAction::Initialize(
                    word.clone(),
                    words,
                    word_set.clone(),
                ));
                worker.send(WordleWorkerInput::Simulation(
                    SimulationInput::StartKnownAnswer {
                        correct: word,
                        guess: None,
                    },
                ));
            } else {
                simulation_state.dispatch(SimulationStateAction::Reset);
                set_toast(ToastOption::new(
                    "No words selected".to_string(),
                    ToastType::Error,
                ));
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

    let on_tab_change = {
        let active_tab = active_tab.clone();
        Callback::from(move |e: MouseEvent| {
            let element: HtmlElement = e.target_unchecked_into();
            if let Some(tab) = element.dataset().get("tab") {
                active_tab.set(Tab::from_str(tab.as_str()).unwrap())
            }
        })
    };

    let running = !simulation_state.words_left.is_empty();
    let all_words_len = all_words.borrow().len();
    let progress_value = all_words_len
        - words_left.borrow().len()
        - if let Some(_) = simulation_state.current_word {
            1
        } else {
            0
        };

    html! {
        <section>
            <div class="columns">
                <div class="column col-2 col-xl-8 col-sm-12 col-mx-auto text-center">
                    <SelectWords dictionary={word_set.dictionary.clone()} {on_words_set} />
                    <button
                        class="btn btn-primary"
                        onclick={on_start_button_click}
                    >{ "Start" }</button>
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
                </div>
            </div>
            <progress class="progress" value={progress_value.to_string()} max={all_words_len.to_string()}/>
            <ul class="tab tab-block" onclick={on_tab_change}>
                {
                    Tab::iter().map(|tab| {
                        html! {
                            <li class={classes!("tab-item", (tab == *active_tab).then(|| "active"))}>
                                <a href={format!("#{tab}")} data-tab={tab.to_string()}>{ tab.to_string() }</a>
                            </li>
                        }
                    }).collect::<Html>()
                }
            </ul>
            {
                match *active_tab {
                    Tab::History => html! {
                        <SimulationHistory
                            history={simulation_state.history.clone()}
                            history_small={simulation_state.history_small.clone()}
                            word_set={word_set.clone()}
                        />
                    },
                    Tab::Detail => html! {
                        <SimulationDetail history={simulation_state.history.clone()} word_set={word_set.clone()} />
                    },
                    Tab::Calibration => {
                        html! {
                        <Calibration
                            data={simulation_state.turns_data.clone()}
                            word_set_name={word_set.name.clone()}
                            used_calibration={word_set.calibration.get_calibration()} />
                        }
                    }
                }
            }
       </section>
    }
}
