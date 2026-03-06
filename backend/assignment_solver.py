import csv
from ortools.linear_solver import pywraplp

def solve_assignments(csv_file_path):
    with open(csv_file_path, mode='r', encoding='utf-8') as f:
        reader = csv.DictReader(f)
        return solve_assignments_from_list(list(reader))

def solve_assignments_from_list(rows):
    # --- Data Cleaning ---
    # Strip whitespace from all fields
    rows = [{k: v.strip() if isinstance(v, str) else v for k, v in row.items()} for row in rows]

    students = []
    projects_set = set()
    all_netids = {row['NetID'] for row in rows}

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

        # Filter self-references and invalid NetIDs from teammates
        teammates = [t for t in teammates if t and t != row['NetID'] and t in all_netids]

        # Handle students with no choices: use pitched project as fallback
        if not any(choices) and pitched:
            choices = [pitched, '', '', '', '']

        student_data = {
            'name': row['Name'],
            'netid': row['NetID'],
            'choices': choices,
            'teammates': teammates,
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

    # Try hard teammate constraint first, fall back to soft-only
    result = _solve_with_constraints(students, projects, project_to_idx, num_students, num_projects, hard_teammate=True)
    if result is None:
        result = _solve_with_constraints(students, projects, project_to_idx, num_students, num_projects, hard_teammate=False)

    if result is None:
        return None

    solver_status, x, y, _solver = result  # keep solver ref alive for solution values
    return _build_results(students, projects, x, y, num_students, num_projects, solver_status)


def _solve_with_constraints(students, projects, project_to_idx, num_students, num_projects, hard_teammate=False):
    """Build and solve the ILP. Returns (status, x, y, solver) or None."""
    netid_to_idx = {s['netid']: i for i, s in enumerate(students)}

    solver = pywraplp.Solver.CreateSolver('SCIP')
    if not solver:
        return None

    # Variables
    x = [[solver.BoolVar(f'x_{s}_{p}') for p in range(num_projects)] for s in range(num_students)]
    y = [solver.BoolVar(f'y_{p}') for p in range(num_projects)]

    # Constraints: Single Assignment
    for s in range(num_students):
        solver.Add(sum(x[s][p] for p in range(num_projects)) == 1)

    # Constraints: Team Capacity (4-6 members)
    for p in range(num_projects):
        solver.Add(sum(x[s][p] for s in range(num_students)) <= 6 * y[p])
        solver.Add(sum(x[s][p] for s in range(num_students)) >= 4 * y[p])

    # Constraints: Pitcher Requirement
    for s_idx, student in enumerate(students):
        if student['is_pitcher']:
            p_name = student['choices'][0]
            if p_name in project_to_idx:
                p_idx = project_to_idx[p_name]
                solver.Add(x[s_idx][p_idx] == y[p_idx])

    # Constraints: Hard teammate (at least one preferred teammate on same team)
    if hard_teammate:
        for s_idx, student in enumerate(students):
            valid_mates = [t for t in student['teammates'] if t in netid_to_idx]
            if not valid_mates:
                continue
            match_vars = []
            for t_netid in valid_mates:
                t_idx = netid_to_idx[t_netid]
                for p_idx in range(num_projects):
                    m = solver.BoolVar(f'm_{s_idx}_{t_idx}_{p_idx}')
                    solver.Add(m <= x[s_idx][p_idx])
                    solver.Add(m <= x[t_idx][p_idx])
                    solver.Add(m >= x[s_idx][p_idx] + x[t_idx][p_idx] - 1)
                    match_vars.append(m)
            solver.Add(sum(match_vars) >= 1)

    # Objective
    weights = [0, 5, 15, 30, 50]
    unlisted_penalty = 200
    teammate_penalty_weight = 50
    obj_terms = []

    for s_idx, student in enumerate(students):
        for p_idx, p_name in enumerate(projects):
            if p_name in student['choices']:
                choice_rank = student['choices'].index(p_name)
                obj_terms.append(x[s_idx][p_idx] * weights[choice_rank])
            else:
                obj_terms.append(x[s_idx][p_idx] * unlisted_penalty)

    # Teammate Penalty: z_{s,t,p} >= x[s][p] - x[t][p] and z_{s,t,p} >= x[t][p] - x[s][p]
    netid_to_idx = {s['netid']: i for i, s in enumerate(students)}
    for s_idx, student in enumerate(students):
        for t_netid in student['teammates']:
            if t_netid in netid_to_idx:
                t_idx = netid_to_idx[t_netid]
                if s_idx < t_idx: # Avoid double counting
                    for p_idx in range(num_projects):
                        z = solver.BoolVar(f'z_{s_idx}_{t_idx}_{p_idx}')
                        solver.Add(z >= x[s_idx][p_idx] - x[t_idx][p_idx])
                        solver.Add(z >= x[t_idx][p_idx] - x[s_idx][p_idx])
                        obj_terms.append(z * teammate_penalty_weight)

    solver.Minimize(solver.Sum(obj_terms))
    solver.SetTimeLimit(60000)
    status = solver.Solve()

    if status == pywraplp.Solver.OPTIMAL:
        return ('OPTIMAL', x, y, solver)
    elif status == pywraplp.Solver.FEASIBLE:
        return ('FEASIBLE', x, y, solver)
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
