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

    // Teammate Requirements
    let netid_to_idx: HashMap<String, usize> = students.iter().enumerate()
        .map(|(i, s)| (s.netid.clone(), i))
        .collect();

    for (s_idx, student) in students.iter().enumerate() {
        let valid_teammate_indices: Vec<usize> = student.teammates.iter()
            .filter_map(|id| netid_to_idx.get(id).copied())
            .filter(|&t_idx| t_idx != s_idx)
            .collect();

        if !valid_teammate_indices.is_empty() {
            for p in 0..num_projects {
                // To satisfy "at least one preferred teammate", we create a variable z
                // that is true if student s is with at least one teammate t on project p.
                // However, the spec is simpler: if s is on p, then at least one t must be on p.
                // sum(x[t][p] for t in teammates) >= x[s][p]
                let teammate_sum: Expression = valid_teammate_indices.iter()
                    .map(|&t_idx| Expression::from(x[t_idx][p]))
                    .sum();
                constraints.push(constraint!(teammate_sum >= x[s_idx][p]));
            }
        }
    }

    // Team size preference: Prefer 6 over 5 or 4.
    for p in 0..num_projects {
        let is_size_6 = vars.add(variable().binary());
        let col_sum: Expression = (0..num_students).map(|s| Expression::from(x[s][p])).sum();
        // is_size_6 can only be 1 if col_sum is 6
        constraints.push(constraint!(col_sum >= 6.0 * is_size_6));
        objective -= is_size_6 * 1.0; // Small incentive for size 6
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> SolverConfig {
        SolverConfig {
            name_col: "Name".to_string(),
            netid_col: "NetID".to_string(),
            pitcher_col: "Project Pitched".to_string(),
            preference_cols: vec![
                "Choice 1".to_string(),
                "Choice 2".to_string(),
            ],
            teammate_cols: vec!["Teammate".to_string()],
            min_team_size: 2,
            max_team_size: 3,
            weights: vec![0.0, 10.0],
            unlisted_penalty: 100.0,
            teammate_penalty: 50.0,
        }
    }

    #[test]
    fn test_basic_assignment() {
        let config = create_test_config();
        let mut data = Vec::new();
        
        let students = vec![
            ("S1", "n1", "P1", "P2"),
            ("S2", "n2", "P1", "P2"),
            ("S3", "n3", "P2", "P1"),
            ("S4", "n4", "P2", "P1"),
        ];

        for (name, id, c1, c2) in students {
            let mut row = HashMap::new();
            row.insert("Name".to_string(), name.to_string());
            row.insert("NetID".to_string(), id.to_string());
            row.insert("Choice 1".to_string(), c1.to_string());
            row.insert("Choice 2".to_string(), c2.to_string());
            data.push(row);
        }

        let result = solve(&config, &data).expect("Should find a solution");
        assert_eq!(result.len(), 2);
        assert!(result.contains_key("P1"));
        assert!(result.contains_key("P2"));
        assert_eq!(result.get("P1").unwrap().len(), 2);
        assert_eq!(result.get("P2").unwrap().len(), 2);
    }

    #[test]
    fn test_pitcher_constraint() {
        let mut config = create_test_config();
        config.min_team_size = 2;
        
        let mut data = Vec::new();
        // S1 pitches P1. P1 MUST be active and S1 MUST be in it.
        let mut row1 = HashMap::new();
        row1.insert("Name".to_string(), "S1".to_string());
        row1.insert("NetID".to_string(), "n1".to_string());
        row1.insert("Project Pitched".to_string(), "P1".to_string());
        row1.insert("Choice 1".to_string(), "P1".to_string());
        data.push(row1);

        let mut row2 = HashMap::new();
        row2.insert("Name".to_string(), "S2".to_string());
        row2.insert("NetID".to_string(), "n2".to_string());
        row2.insert("Choice 1".to_string(), "P1".to_string());
        data.push(row2);

        let result = solve(&config, &data).expect("Should solve");
        assert!(result.contains_key("P1"));
        let members = result.get("P1").unwrap();
        assert!(members.contains(&"S1".to_string()));
    }

    #[test]
    fn test_full_dataset() {
        let csv_text = include_str!("../backend/GenAI-InputFile - ProjectPreferences.csv");
        let (_headers, data) = crate::csv_parser::parse_csv(csv_text).expect("Should parse CSV");

        let config = SolverConfig {
            name_col: "Name".to_string(),
            netid_col: "NetID".to_string(),
            pitcher_col: "Project Pitched".to_string(),
            preference_cols: vec![
                "First (1) Choice".to_string(),
                "Second (2)  Choice".to_string(),
                "Third (3) Choice".to_string(),
                "Fourth (4) Choice".to_string(),
                "Fifth (5) Choice".to_string(),
            ],
            teammate_cols: vec![
                "Team Member #1 UW NetID".to_string(),
                "Team Member #2 UW NetID".to_string(),
                "Team Member #3 UW NetID".to_string(),
            ],
            min_team_size: 4,
            max_team_size: 6,
            weights: vec![0.0, 5.0, 15.0, 30.0, 50.0],
            unlisted_penalty: 100.0,
            teammate_penalty: 50.0,
        };

        let result = solve(&config, &data);
        assert!(result.is_some(), "Should find a feasible solution for the full dataset");
        
        let assignments = result.unwrap();
        let total_assigned: usize = assignments.values().map(|m| m.len()).sum();
        assert_eq!(total_assigned, data.len(), "Every student should be assigned to a project");
    }
}
