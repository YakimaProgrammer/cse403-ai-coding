use yew::prelude::*;
use gloo_file::callbacks::read_as_text;
use gloo_file::File;
use web_sys::HtmlInputElement;
use std::collections::HashMap;

use crate::solver::{solve, SolverConfig};
use crate::csv_parser::parse_csv;

#[function_component(App)]
pub fn app() -> Html {
    let csv_data = use_state(|| Vec::<HashMap<String, String>>::new());
    let columns = use_state(|| Vec::<String>::new());
    let result = use_state(|| None::<HashMap<String, Vec<String>>>);

    // Configuration State
    let name_col = use_state(|| "".to_string());
    let netid_col = use_state(|| "".to_string());
    let pitcher_col = use_state(|| "".to_string());
    let pref_cols = use_state(|| vec!["".to_string()]);
    let teammate_cols = use_state(|| vec!["".to_string()]);
    let min_size = use_state(|| 4u32);
    let max_size = use_state(|| 6u32);

    let on_file_change = {
        let csv_data = csv_data.clone();
        let columns = columns.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    let file = File::from(file);
                    let csv_data = csv_data.clone();
                    let columns = columns.clone();
                    read_as_text(&file, move |res| {
                        if let Ok(text) = res {
                            if let Ok((headers, data)) = parse_csv(&text) {
                                columns.set(headers);
                                csv_data.set(data);
                            }
                        }
                    });
                }
            }
        })
    };

    let on_solve = {
        let csv_data = csv_data.clone();
        let result = result.clone();
        let name_col = name_col.clone();
        let netid_col = netid_col.clone();
        let pitcher_col = pitcher_col.clone();
        let pref_cols = pref_cols.clone();
        let teammate_cols = teammate_cols.clone();
        let min_size = min_size.clone();
        let max_size = max_size.clone();

        Callback::from(move |_| {
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

            let solve_result = solve(&config, &csv_data);
            result.set(solve_result);
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
                        <select onchange={let name_col = name_col.clone(); Callback::from(move |e: Event| name_col.set(e.target_unchecked_into::<web_sys::HtmlSelectElement>().value()))}>
                            <option value="">{ "-- Select --" }</option>
                            { for columns.iter().map(|col| html! { <option value={col.clone()}>{ col }</option> }) }
                        </select>
                    </div>
                    <div>
                        <label>{ "NetID Column: " }</label>
                        <select onchange={let netid_col = netid_col.clone(); Callback::from(move |e: Event| netid_col.set(e.target_unchecked_into::<web_sys::HtmlSelectElement>().value()))}>
                            <option value="">{ "-- Select --" }</option>
                            { for columns.iter().map(|col| html! { <option value={col.clone()}>{ col }</option> }) }
                        </select>
                    </div>
                    <div>
                        <label>{ "Pitcher Column: " }</label>
                        <select onchange={let pitcher_col = pitcher_col.clone(); Callback::from(move |e: Event| pitcher_col.set(e.target_unchecked_into::<web_sys::HtmlSelectElement>().value()))}>
                            <option value="">{ "-- Select --" }</option>
                            { for columns.iter().map(|col| html! { <option value={col.clone()}>{ col }</option> }) }
                        </select>
                    </div>

                    <h3>{ "Settings" }</h3>
                    <label>{ "Min Team Size: " }</label>
                    <input type="number" value={min_size.to_string()} onchange={let min_size = min_size.clone(); Callback::from(move |e: Event| min_size.set(e.target_unchecked_into::<HtmlInputElement>().value().parse().unwrap_or(4)))} />
                    <label>{ " Max Team Size: " }</label>
                    <input type="number" value={max_size.to_string()} onchange={let max_size = max_size.clone(); Callback::from(move |e: Event| max_size.set(e.target_unchecked_into::<HtmlInputElement>().value().parse().unwrap_or(6)))} />

                    <div style="margin-top: 20px;">
                        <button onclick={on_solve}>{ "Solve Assignments" }</button>
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
