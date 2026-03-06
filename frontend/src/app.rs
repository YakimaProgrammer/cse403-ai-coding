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
        Callback::from(move |_| {
            let config = SolverConfig {
                name_col: "Name".to_string(),
                netid_col: "NetID".to_string(),
                pitcher_col: "Project Pitched".to_string(),
                preference_cols: vec![
                    "First (1) Choice".to_string(),
                    "Second (2)  Choice".to_string(),
                    "Third (3) Choice".to_string(),
                ],
                teammate_cols: vec![
                    "Team Member #1 UW NetID".to_string(),
                ],
                min_team_size: 4,
                max_team_size: 6,
                weights: vec![0.0, 5.0, 15.0],
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
            <input type="file" accept=".csv" onchange={on_file_change} />
            
            if !csv_data.is_empty() {
                <button onclick={on_solve}>{ "Solve Assignments" }</button>
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
