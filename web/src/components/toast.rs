use bounce::{use_atom, Atom};
use gloo_timers::callback::Timeout;
use yew::{classes, function_component, html, use_effect, Callback};

#[allow(dead_code)]
#[derive(PartialEq)]
pub enum ToastType {
    Info,
    Success,
    Warning,
    Error,
}

pub struct Toast {
    message: String,
    t_type: ToastType,
}

impl PartialEq for Toast {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

#[derive(Default, PartialEq, Atom)]
pub struct ToastOption(pub Option<Toast>);

impl ToastOption {
    pub fn new(message: String, t_type: ToastType) -> Self {
        Self(Some(Toast { message, t_type }))
    }

    pub fn none() -> Self {
        Self(None)
    }
}

#[function_component(ToastComponent)]
pub fn view() -> Html {
    let toast_atom = use_atom::<ToastOption>();

    if let Some(toast) = &toast_atom.0 {
        {
            let toast_atom = toast_atom.clone();
            use_effect(move || {
                let timeout = {
                    let toast_atom = toast_atom.clone();
                    Timeout::new(3_000, move || toast_atom.set(ToastOption::none()))
                };
                || {
                    timeout.cancel();
                }
            })
        }

        let toast_class = match toast.t_type {
            ToastType::Info => "toast-primary",
            ToastType::Success => "toast-success",
            ToastType::Warning => "toast-warning",
            ToastType::Error => "toast-error",
        };

        let onclick = {
            let toast_atom = toast_atom.clone();
            Callback::from(move |_| {
                toast_atom.set(ToastOption::none());
            })
        };

        html! {
            <div class="container p-absolute">
                <div class="columns">
                    <div class="column col-5 col-xl-4 hide-md"/>
                    <div style="padding-right: 0px" class="column col-2 col-xl-4 col-md-12">
                        <div class={classes!("toast", toast_class)}>
                            <button class="btn btn-clear float-right" {onclick}></button>
                            { &toast.message }
                        </div>
                    </div>
                    <div class="column col-5 col-xl-4 hide-md"/>
                </div>
            </div>
        }
    } else {
        html! { <> </> }
    }
}
