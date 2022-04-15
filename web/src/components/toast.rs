use bounce::{use_atom, Atom};
use yew::{classes, function_component, html, Callback};

#[derive(PartialEq)]
pub enum ToastType {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(PartialEq)]
pub struct Toast {
    message: String,
    t_type: ToastType,
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
            <div class={classes!("toast", toast_class)}>
                <button class="btn btn-clear float-right" {onclick}></button>
                { &toast.message }
            </div>
        }
    } else {
        html! { <> </> }
    }
}
