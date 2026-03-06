use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use good_lp::{variables, variable, Expression, SolverModel, Solution, constraint};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SolverConfig {
    pub name_col: String,
    pub netid_col: String,
    pub pitcher_col: String,
    pub preference_cols: Vec<String>,
    pub teammate_cols: Vec<String>,
    pub min_team_size: u32,
    pub max_team_size: u32,
    pub weights: Vec<f64>, // e.g., [0.0, 5.0, 15.0, 30.0, 50.0]
    pub unlisted_penalty: f64,
    pub teammate_penalty: f64,
}

#[derive(Debug, Clone)]
pub struct Student {
    pub name: String,
    pub netid: String,
    pub choices: Vec<String>,
    pub teammates: Vec<String>,
    pub is_pitcher: bool,
}

pub fn solve(config: &SolverConfig, raw_data: &[HashMap<String, String>]) -> Option<HashMap<String, Vec<String>>> {
    // 1. Pre-process Data
    let mut students = Vec::new();
    let mut projects_set = HashSet::new();

    for row in raw_data {
        let pitched = row.get(&config.pitcher_col).cloned().unwrap_or_default();
        let choices: Vec<String> = config.preference_cols.iter()
            .filter_map(|col| row.get(col).filter(|s| !s.is_empty()).cloned())
            .collect();
        
        let is_pitcher = !pitched.is_empty() && !choices.is_empty() && pitched == choices[0];
        
        let student = Student {
            name: row.get(&config.name_col).cloned().unwrap_or_default(),
            netid: row.get(&config.netid_col).cloned().unwrap_or_default(),
            teammates: config.teammate_cols.iter()
                .filter_map(|col| row.get(col).filter(|s| !s.is_empty()).cloned())
                .collect(),
            is_pitcher,
            choices: choices.clone(),
        };
        
        for choice in &choices { projects_set.insert(choice.clone()); }
        students.push(student);
    }

    let mut projects: Vec<String> = projects_set.into_iter().collect();
    projects.sort();
    let num_students = students.len();
    let num_projects = projects.len();

    // 2. Setup Solver
    let mut vars = variables!();
    
    // x[s][p] matrix: student s assigned to project p
    let x: Vec<Vec<_>> = (0..num_students)
        .map(|_| (0..num_projects).map(|_| vars.add(variable().binary())).collect())
        .collect();

    // y[p]: project p is active
    let y: Vec<_> = (0..num_projects).map(|_| vars.add(variable().binary())).collect();

    // 3. Constraints
    let mut constraints = Vec::new();

    // Each student assigned to exactly one project
    for s in 0..num_students {
        let row_sum: Expression = x[s].iter().map(|&v| Expression::from(v)).sum();
        constraints.push(constraint!(row_sum == 1.0));
    }

    // Team Size Constraints
    for p in 0..num_projects {
        let col_sum: Expression = (0..num_students).map(|s| Expression::from(x[s][p])).sum();
        constraints.push(constraint!(col_sum.clone() <= (config.max_team_size as f64) * y[p]));
        constraints.push(constraint!(col_sum >= (config.min_team_size as f64) * y[p]));
    }

    // Pitcher Logic
    for (s_idx, student) in students.iter().enumerate() {
        if student.is_pitcher {
            if let Some(p_idx) = projects.iter().position(|p| p == &student.choices[0]) {
                constraints.push(constraint!(x[s_idx][p_idx] == y[p_idx]));
            }
        }
    }

    // 4. Objective (Weights)
    let mut objective = Expression::from(0.0);
    for s in 0..num_students {
        for p in 0..num_projects {
            let weight = if let Some(rank) = students[s].choices.iter().position(|c| c == &projects[p]) {
                *config.weights.get(rank).unwrap_or(&config.unlisted_penalty)
            } else {
                config.unlisted_penalty
            };
            objective += x[s][p] * weight;
        }
    }

    // Teammate Penalties
    let netid_to_idx: HashMap<String, usize> = students.iter().enumerate()
        .map(|(i, s)| (s.netid.clone(), i))
        .collect();

    for (s_idx, student) in students.iter().enumerate() {
        for t_netid in &student.teammates {
            if let Some(&t_idx) = netid_to_idx.get(t_netid) {
                if s_idx < t_idx {
                    for p in 0..num_projects {
                        let z = vars.add(variable().binary());
                        // z >= x[s][p] - x[t][p]
                        constraints.push(constraint!(z >= x[s_idx][p] - x[t_idx][p]));
                        // z >= x[t][p] - x[s][p]
                        constraints.push(constraint!(z >= x[t_idx][p] - x[s_idx][p]));
                        
                        objective += z * config.teammate_penalty;
                    }
                }
            }
        }
    }

    // 5. Solve and Format
    let mut model = vars.minimise(objective).using(good_lp::microlp);
    for c in constraints {
        model = model.with(c);
    }

    if let Ok(solution) = model.solve() {
        let mut result = HashMap::new();
        for p in 0..num_projects {
            if solution.value(y[p]) > 0.5 {
                let members: Vec<String> = (0..num_students)
                    .filter(|&s| solution.value(x[s][p]) > 0.5)
                    .map(|s| students[s].name.clone())
                    .collect();
                result.insert(projects[p].clone(), members);
            }
        }
        Some(result)
    } else {
        None
    }
}
