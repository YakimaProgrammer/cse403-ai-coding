use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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
    let mut students = Vec::new();
    let mut project_names = HashSet::new();

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
        
        for choice in &choices { project_names.insert(choice.clone()); }
        students.push(student);
    }

    let projects: Vec<String> = project_names.into_iter().collect();
    let num_students = students.len();

    // Simplified Greedy Assignment for WASM performance
    let mut assignments: HashMap<String, Vec<usize>> = HashMap::new();
    let mut student_to_project = vec![None; num_students];

    // 1. Assign Pitchers first
    for (s_idx, student) in students.iter().enumerate() {
        if student.is_pitcher {
            let p_name = &student.choices[0];
            assignments.entry(p_name.clone()).or_default().push(s_idx);
            student_to_project[s_idx] = Some(p_name.clone());
        }
    }

    // 2. Assign remaining students to their top choices if possible
    for (s_idx, student) in students.iter().enumerate() {
        if student_to_project[s_idx].is_some() { continue; }
        
        for choice in &student.choices {
            let count = assignments.get(choice).map(|v| v.len()).unwrap_or(0);
            if count < config.max_team_size as usize {
                assignments.entry(choice.clone()).or_default().push(s_idx);
                student_to_project[s_idx] = Some(choice.clone());
                break;
            }
        }
    }

    // 3. Fill unassigned students into projects with space
    for (s_idx, _) in students.iter().enumerate() {
        if student_to_project[s_idx].is_some() { continue; }
        for p_name in &projects {
            let count = assignments.get(p_name).map(|v| v.len()).unwrap_or(0);
            if count < config.max_team_size as usize {
                assignments.entry(p_name.clone()).or_default().push(s_idx);
                student_to_project[s_idx] = Some(p_name.clone());
                break;
            }
        }
    }

    // Filter projects that don't meet minimum size
    let result: HashMap<String, Vec<String>> = assignments.into_iter()
        .filter(|(_, members)| members.len() >= config.min_team_size as usize)
        .map(|(p_name, members)| {
            (p_name, members.into_iter().map(|idx| students[idx].name.clone()).collect())
        })
        .collect();

    if result.values().map(|v| v.len()).sum::<usize>() == num_students {
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
