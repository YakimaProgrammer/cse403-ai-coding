import csv
from ortools.linear_solver import pywraplp

def solve_assignments(csv_file_path):
    with open(csv_file_path, mode='r', encoding='utf-8') as f:
        reader = csv.DictReader(f)
        return solve_assignments_from_list(list(reader))

def solve_assignments_from_list(rows):
    # Strip whitespace from keys and values
    rows = [{k.strip(): v.strip() if isinstance(v, str) else v for k, v in row.items()} for row in rows]
    students = []
    projects_set = set()
    
    for row in rows:
        choices = [
            row['First (1) Choice'], 
            row['Second (2)  Choice'], 
            row['Third (3) Choice'], 
            row['Fourth (4) Choice'], 
            row['Fifth (5) Choice']
        ]
        teammates = [
            row['Team Member #1 UW NetID'],
            row['Team Member #2 UW NetID'],
            row['Team Member #3 UW NetID']
        ]
        pitched = row['Project Pitched']
        student_data = {
            'name': row['Name'],
            'netid': row['NetID'],
            'choices': choices,
            'teammates': [t for t in teammates if t],
            'is_pitcher': pitched and choices and (len(choices) > 0 and pitched == choices[0])
        }
        students.append(student_data)
        for choice in student_data['choices']:
            if choice:
                projects_set.add(choice)

    projects = sorted(list(projects_set))
    project_to_idx = {p: i for i, p in enumerate(projects)}
    num_students = len(students)
    num_projects = len(projects)

    solver = pywraplp.Solver.CreateSolver('SCIP')
    if not solver:
        return None

    # Variables
    x = [[solver.BoolVar(f'x_{s}_{p}') for p in range(num_projects)] for s in range(num_students)]
    y = [solver.BoolVar(f'y_{p}') for p in range(num_projects)]

    # Constraints: Single Assignment
    for s in range(num_students):
        solver.Add(sum(x[s][p] for p in range(num_projects)) == 1)

    # Objective initialization
    obj_terms = []

    # Constraints: Team Capacity
    for p in range(num_projects):
        team_size = sum(x[s][p] for s in range(num_students))
        solver.Add(team_size <= 6 * y[p])
        solver.Add(team_size >= 4 * y[p])
        
        # Team Size Penalties/Rewards
        is_size_6 = solver.BoolVar(f'is_size_6_{p}')
        is_size_5 = solver.BoolVar(f'is_size_5_{p}')
        is_size_4 = solver.BoolVar(f'is_size_4_{p}')

        # Ensure exactly one size variable is true if the project is active
        solver.Add(is_size_6 + is_size_5 + is_size_4 == y[p])
        
        # Link size variables to actual team_size
        solver.Add(team_size >= 6 * is_size_6 + 5 * is_size_5 + 4 * is_size_4)
        solver.Add(team_size <= 6 * is_size_6 + 5 * is_size_5 + 4 * is_size_4 + 6 * (1 - y[p]))

        # Penalties: 6 (reward -1), 5 (penalty 25), 4 (penalty 50)
        obj_terms.append(is_size_6 * -1)
        obj_terms.append(is_size_5 * 25)
        obj_terms.append(is_size_4 * 50)

    # Constraints: Pitcher Requirement
    for s_idx, student in enumerate(students):
        if student['is_pitcher']:
            p_name = student['choices'][0]
            if p_name in project_to_idx:
                p_idx = project_to_idx[p_name]
                solver.Add(x[s_idx][p_idx] == y[p_idx])

    # Objective weights
    weights = [0, 5, 15, 30, 50]
    unlisted_penalty = 200

    for s_idx, student in enumerate(students):
        for p_idx, p_name in enumerate(projects):
            if p_name in student['choices']:
                choice_rank = student['choices'].index(p_name)
                obj_terms.append(x[s_idx][p_idx] * weights[choice_rank])
            else:
                obj_terms.append(x[s_idx][p_idx] * unlisted_penalty)

    # Teammate Requirement: Penalize for each teammate preference not honored
    teammate_penalty = 50
    netid_to_idx = {s['netid']: i for i, s in enumerate(students)}
    for s_idx, student in enumerate(students):
        valid_mates = [netid_to_idx[t] for t in student['teammates'] if t in netid_to_idx and netid_to_idx[t] != s_idx]
        if valid_mates:
            # Soft constraint: penalize if not with ANY preferred teammate
            is_alone = solver.BoolVar(f'alone_{s_idx}')
            for p_idx in range(num_projects):
                # if x[s][p] and sum(x[t][p]) == 0, then is_alone must be 1
                solver.Add(is_alone >= x[s_idx][p_idx] - sum(x[t_idx][p_idx] for t_idx in valid_mates))
            obj_terms.append(is_alone * teammate_penalty)

    solver.Minimize(solver.Sum(obj_terms))

    status = solver.Solve()

    if status == pywraplp.Solver.OPTIMAL:
        return _build_results(students, projects, x, y, num_students, num_projects, 'OPTIMAL')
    elif status == pywraplp.Solver.FEASIBLE:
        return _build_results(students, projects, x, y, num_students, num_projects, 'FEASIBLE')
    else:
        return None


def _build_results(students, projects, x, y, num_students, num_projects, solver_status):
    """Build enriched results with team assignments, metrics, and student details."""
    netid_to_idx = {s['netid']: i for i, s in enumerate(students)}

    # Build team assignments
    teams = {}
    student_assignments = {}  # s_idx -> p_idx
    for p_idx, p_name in enumerate(projects):
        if y[p_idx].solution_value() > 0.5:
            members = []
            for s_idx in range(num_students):
                if x[s_idx][p_idx].solution_value() > 0.5:
                    members.append(students[s_idx]['name'])
                    student_assignments[s_idx] = p_idx
            if members:
                teams[p_name] = members

    # Per-student details and choice distribution
    choice_distribution = {"1": 0, "2": 0, "3": 0, "4": 0, "5": 0, "unlisted": 0}
    student_details = []

    for s_idx, student in enumerate(students):
        assigned_p_idx = student_assignments.get(s_idx)
        if assigned_p_idx is None:
            continue
        assigned_project = projects[assigned_p_idx]

        # Determine choice rank
        if assigned_project in student['choices']:
            rank = student['choices'].index(assigned_project) + 1
            choice_distribution[str(rank)] += 1
        else:
            rank = 6
            choice_distribution["unlisted"] += 1

        # Check teammate satisfaction
        has_teammate = any(
            student_assignments.get(netid_to_idx[t]) == assigned_p_idx
            for t in student['teammates']
            if t in netid_to_idx
        )

        student_details.append({
            'name': student['name'],
            'netid': student['netid'],
            'assigned_project': assigned_project,
            'choice_rank': rank,
            'has_preferred_teammate': has_teammate,
        })

    # Teammate satisfaction stats
    total_with_prefs = sum(1 for s in students if len(s['teammates']) > 0)
    satisfied_with_prefs = sum(
        1 for sd in student_details
        if sd['has_preferred_teammate'] and
        len(students[netid_to_idx[sd['netid']]]['teammates']) > 0
    )

    team_sizes = {name: len(members) for name, members in teams.items()}

    return {
        'teams': teams,
        'metrics': {
            'choice_distribution': choice_distribution,
            'teammate_satisfaction': {
                'total_with_preferences': total_with_prefs,
                'satisfied': satisfied_with_prefs,
                'percentage': round(satisfied_with_prefs / total_with_prefs * 100, 1) if total_with_prefs > 0 else 0,
            },
            'team_sizes': team_sizes,
            'num_teams': len(teams),
            'solver_status': solver_status,
        },
        'student_details': student_details,
    }
