use dioxus::prelude::*;
use dioxus_sdk::storage::use_persistent;
use std::path::Path;
use tracing::Level;

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
  #[route("/")]
  Home {},
  #[route("/config")]
  Config,
}

mod data;

fn main() {
  dioxus_sdk::set_dir!();
  // Init logger
  dioxus_logger::init(Level::INFO).expect("failed to init logger");

  let cfg = dioxus::desktop::Config::new()
    .with_custom_head(r#"<link rel="stylesheet" href="app.css">"#.to_string());
  LaunchBuilder::desktop().with_cfg(cfg).launch(App);
}

#[component]
fn App() -> Element {
  let first_run = use_persistent("first_run", || true);
  let initial_route = if first_run() {
    Route::Config
  } else {
    Route::Home {}
  };
  rsx! {
    body {
      Router::<Route> {
        config: || RouterConfig::default().initial_route(initial_route)
      }
    }
  }
}

#[component]
fn Home() -> Element {
  rsx! {
    main {
      class: "container",
      nav {
        ul { li { span { class: "header", "YNOManager" } } }
        ul {
          li { Link { to: Route::Config, "Settings" } }
        }
      }
      div {
        role: "group",
        button { "Update All" }
      }
      table {
        thead { tr {
          th { "Name" }
          th { "Installed" }
          th { "Latest" }
          th {}
        } }
        tbody {
          for _ in 0..20 {
            tr {
              td { "Yume 2kki" }
              td { "0.123 patch 4" }
              td { "0.123 patch 5" }
              td {
                Link { to: "install", "Install" }
                span { " | " }
                Link { to: "update", "Update" }
                span { " (" }
                Link { to: "select-version", "to version..."}
                span { ") | " }
                Link { to: "uninstall", "Uninstall" }
              }
            }
          }
        }
      }
    }
  }
}

#[derive(Default, Clone, Copy)]
enum ValidationState {
  #[default]
  Unknown,
  Invalid,
}

impl ValidationState {
  fn aria_invalid(&self) -> &str {
    match self {
      ValidationState::Unknown => "",
      ValidationState::Invalid => "true",
    }
  }
  fn is_valid(&self) -> bool {
    !matches!(self, ValidationState::Invalid)
  }
}

#[component]
fn Config() -> Element {
  let nav = navigator();
  let mut first_run = use_persistent("first_run", || true);
  let mut root_path = use_persistent("root_path", String::new);
  let mut root_path_valid = use_signal(|| ValidationState::default());

  rsx! {
    main {
      class: "container",
      if first_run() {
        hgroup {
            h4 { "Welcome" }
            p { "Let's configure some settings first." }
        }
      } else {
        h4 { "Settings" }
      }
      form {
        action: "#",
        onsubmit: move |e| {
          let form = e.data().values();
          root_path_valid.set(ValidationState::Unknown);
          let Some(path) = form.get("root_path") else {
            root_path_valid.set(ValidationState::Invalid);
            return;
          };
          let path = path.first().map(String::as_str).unwrap_or_default();
          if !Path::new(&path).is_dir() {
            root_path_valid.set(ValidationState::Invalid);
          }
          if !root_path_valid().is_valid() {
            return;
          }

          root_path.set(dbg!(path.to_string()));
          first_run.set(false);
          spawn(async move {
            // yield, otherwise the persistent layer has no time to save the data
            tokio::task::yield_now().await;
            nav.replace(Route::Home {});
          });
        },
        fieldset {
          role: "group",
          legend { "Root path" }
          input {
            id: "root_path",
            name: "root_path",
            placeholder: "/path/to/root",
            initial_value: "{root_path}",
            "aria-invalid": root_path_valid().aria_invalid(),
            "aria-describedby": "invalid_root_path"
          }
          button {
            prevent_default: "onclick",
            onclick: move |_| {
              eval("window.root_path_picker.click()");
            },
            "Browse"
          }
        }
        if !root_path_valid().is_valid() {
          small {
            id: "invalid_root_path",
            "Path is empty or does not point to a folder."
          }
        }
        input {
          id: "root_path_picker",
          r#type: "file",
          hidden: true,
          directory: true,
          prevent_default: "onchange",
          onchange: move |e| {
            let Some(files) = e.data().files() else {
                return;
            };
            let files = files.files();
            let Some(file) = files.first() else {
                return;
            };
            root_path.set(file.clone());
            eval(&format!("window.root_path.value={file:?}"));
          }
        }
        input { r#type: "submit", value: "Save" }
      }
    }
  }
}
