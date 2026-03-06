use yew::prelude::*;
use gloo_file::File;
use gloo_file::futures::read_as_text;
use web_sys::HtmlInputElement;
use std::collections::HashMap;
use gloo_worker::{Spawnable, Worker};
use serde::{Deserialize, Serialize};

use crate::solver::{solve, SolverConfig};
use crate::csv_parser::parse_csv;

#[derive(Serialize, Deserialize, Debug)]
pub struct SolverInput {
    pub config: SolverConfig,
    pub csv_data: Vec<HashMap<String, String>>,
}

pub struct SolverWorker;
impl Worker for SolverWorker {
    type Input = SolverInput;
    type Message = ();
    type Output = Option<HashMap<String, Vec<String>>>;

    fn create(_scope: &gloo_worker::WorkerScope<Self>) -> Self {
        Self
    }

    fn update(&mut self, _scope: &gloo_worker::WorkerScope<Self>, _msg: Self::Message) {}

    fn received(&mut self, scope: &gloo_worker::WorkerScope<Self>, msg: Self::Input, id: gloo_worker::HandlerId) {
        let result = solve(&msg.config, &msg.csv_data);
        scope.respond(id, result);
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let csv_data = use_state(|| Vec::<HashMap<String, String>>::new());
    let columns = use_state(|| Vec::<String>::new());
    let result = use_state(|| None::<HashMap<String, Vec<String>>>);
    let is_solving = use_state(|| false);

    let worker_bridge = {
        let result = result.clone();
        let is_solving = is_solving.clone();
        use_memo(
            (),
            move |_| {
                SolverWorker::spawner()
                    .callback(move |res| {
                        result.set(res);
                        is_solving.set(false);
                    })
                    .spawn("/worker.js")
            },
        )
    };

    // Configuration State
    let name_col = use_state(|| "".to_string());
    let netid_col = use_state(|| "".to_string());
    let pitcher_col = use_state(|| "".to_string());
    let pref_cols = use_state(|| vec!["".to_string(); 5]);
    let teammate_cols = use_state(|| vec!["".to_string(); 3]);
    let min_size = use_state(|| 4u32);
    let max_size = use_state(|| 6u32);

    let on_file_change = {
        let csv_data = csv_data.clone();
        let columns = columns.clone();
        Callback::from(move |e: Event| {
            web_sys::console::log_1(&"File input changed".into());
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(files) = input.files() {
                web_sys::console::log_1(&format!("Files found: {}", files.length()).into());
                if let Some(file) = files.get(0) {
                    let file = File::from(file);
                    web_sys::console::log_1(&format!("Processing file: {}", file.name()).into());
                    let csv_data = csv_data.clone();
                    let columns = columns.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        web_sys::console::log_1(&"Starting async file read".into());
                        match read_as_text(&file).await {
                            Ok(text) => {
                                web_sys::console::log_1(&format!("File text read successfully ({} chars)", text.len()).into());
                                match parse_csv(&text) {
                                    Ok((headers, data)) => {
                                        web_sys::console::log_1(&"CSV parsed successfully".into());
                                        columns.set(headers);
                                        csv_data.set(data);
                                    }
                                    Err(e) => web_sys::console::error_1(&format!("CSV Parse Error: {}", e).into()),
                                }
                            }
                            Err(e) => web_sys::console::error_1(&format!("File Read Error: {:?}", e).into()),
                        }
                    });
                }
            }
        })
    };

    let on_solve = {
        let csv_data = csv_data.clone();
        let name_col = name_col.clone();
        let netid_col = netid_col.clone();
        let pitcher_col = pitcher_col.clone();
        let pref_cols = pref_cols.clone();
        let teammate_cols = teammate_cols.clone();
        let min_size = min_size.clone();
        let max_size = max_size.clone();
        let is_solving = is_solving.clone();
        let worker_bridge = worker_bridge.clone();

        Callback::from(move |_| {
            is_solving.set(true);
            let config = SolverConfig {
                name_col: (*name_col).clone(),
                netid_col: (*netid_col).clone(),
                pitcher_col: (*pitcher_col).clone(),
                preference_cols: (*pref_cols).iter().filter(|s| !s.is_empty()).cloned().collect(),
                teammate_cols: (*teammate_cols).iter().filter(|s| !s.is_empty()).cloned().collect(),
                min_team_size: *min_size,
                max_team_size: *max_size,
                weights: vec![0.0, 5.0, 15.0, 30.0, 50.0],
                unlisted_penalty: 100.0,
                teammate_penalty: 50.0,
            };

            worker_bridge.send(SolverInput {
                config,
                csv_data: (*csv_data).clone(),
            });
        })
    };

    html! {
        <main>
            <h1>{ "Project Assignment Solver" }</h1>
            <section>
                <h2>{ "Step 1: Upload Data" }</h2>
                <label>{ "Upload CSV: " }</label>
                <input type="file" accept=".csv" onchange={on_file_change} />
            </section>
            
            if !csv_data.is_empty() {
                <section style="margin-top: 20px; border: 1px solid #ccc; padding: 10px;">
                    <h2>{ "Step 2: Map Columns" }</h2>
                    <p>{ "Please manually map the following fields to the correct columns in your CSV." }</p>
                    <div>
                        <label>{ "Name Column: " }</label>
                        <select value={(*name_col).clone()} onchange={let name_col = name_col.clone(); Callback::from(move |e: Event| name_col.set(e.target_unchecked_into::<web_sys::HtmlSelectElement>().value()))}>
                            <option value="">{ "-- Select --" }</option>
                            { for columns.iter().map(|col| html! { <option value={col.clone()} selected={*name_col == col.clone()}>{ col }</option> }) }
                        </select>
                    </div>
                    <div>
                        <label>{ "NetID Column: " }</label>
                        <select value={(*netid_col).clone()} onchange={let netid_col = netid_col.clone(); Callback::from(move |e: Event| netid_col.set(e.target_unchecked_into::<web_sys::HtmlSelectElement>().value()))}>
                            <option value="">{ "-- Select --" }</option>
                            { for columns.iter().map(|col| html! { <option value={col.clone()} selected={*netid_col == col.clone()}>{ col }</option> }) }
                        </select>
                    </div>
                    <div>
                        <label>{ "Pitcher Column: " }</label>
                        <select value={(*pitcher_col).clone()} onchange={let pitcher_col = pitcher_col.clone(); Callback::from(move |e: Event| pitcher_col.set(e.target_unchecked_into::<web_sys::HtmlSelectElement>().value()))}>
                            <option value="">{ "-- Select --" }</option>
                            { for columns.iter().map(|col| html! { <option value={col.clone()} selected={*pitcher_col == col.clone()}>{ col }</option> }) }
                        </select>
                    </div>

                    <div>
                        <h3>{ "Preference Columns (in order)" }</h3>
                        { for (*pref_cols).iter().enumerate().map(|(i, val)| {
                            let pref_cols = pref_cols.clone();
                            html! {
                                <div key={i}>
                                    <label>{ format!("Choice {}: ", i + 1) }</label>
                                    <select value={val.clone()} onchange={Callback::from(move |e: Event| {
                                        let mut new_prefs = (*pref_cols).clone();
                                        new_prefs[i] = e.target_unchecked_into::<web_sys::HtmlSelectElement>().value();
                                        pref_cols.set(new_prefs);
                                    })}>
                                        <option value="">{ "-- Select --" }</option>
                                        { for columns.iter().map(|col| html! { <option value={col.clone()} selected={val == col}>{ col }</option> }) }
                                    </select>
                                </div>
                            }
                        }) }
                    </div>

                    <div>
                        <h3>{ "Teammate NetID Columns" }</h3>
                        { for (*teammate_cols).iter().enumerate().map(|(i, val)| {
                            let teammate_cols = teammate_cols.clone();
                            html! {
                                <div key={i}>
                                    <label>{ format!("Teammate {}: ", i + 1) }</label>
                                    <select value={val.clone()} onchange={Callback::from(move |e: Event| {
                                        let mut new_teammates = (*teammate_cols).clone();
                                        new_teammates[i] = e.target_unchecked_into::<web_sys::HtmlSelectElement>().value();
                                        teammate_cols.set(new_teammates);
                                    })}>
                                        <option value="">{ "-- Select --" }</option>
                                        { for columns.iter().map(|col| html! { <option value={col.clone()} selected={val == col}>{ col }</option> }) }
                                    </select>
                                </div>
                            }
                        }) }
                    </div>

                    <h3>{ "Settings" }</h3>
                    <label>{ "Min Team Size: " }</label>
                    <input type="number" value={min_size.to_string()} onchange={let min_size = min_size.clone(); Callback::from(move |e: Event| min_size.set(e.target_unchecked_into::<HtmlInputElement>().value().parse().unwrap_or(4)))} />
                    <label>{ " Max Team Size: " }</label>
                    <input type="number" value={max_size.to_string()} onchange={let max_size = max_size.clone(); Callback::from(move |e: Event| max_size.set(e.target_unchecked_into::<HtmlInputElement>().value().parse().unwrap_or(6)))} />

                    <div style="margin-top: 20px;">
                        <button onclick={on_solve} disabled={*is_solving}>
                            { if *is_solving { "Solving..." } else { "Solve Assignments" } }
                        </button>
                    </div>
                </section>
            }

            if let Some(res) = &*result {
                <h2>{ "Results" }</h2>
                <ul>
                    { for res.iter().map(|(project, members)| html! {
                        <li>
                            <strong>{ project }</strong>
                            { ": " }{ members.join(", ") }
                        </li>
                    }) }
                </ul>
            }
        </main>
    }
}
