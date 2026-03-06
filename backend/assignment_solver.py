import csv
from ortools.linear_solver import pywraplp

def solve_assignments(csv_file_path):
    with open(csv_file_path, mode='r', encoding='utf-8') as f:
        reader = csv.DictReader(f)
        return solve_assignments_from_list(list(reader))

def solve_assignments_from_list(rows, config=None):
    if config is None:
        config = {
            'column_map': {
                'name': 'Name',
                'netid': 'NetID',
                'pitched': 'Project Pitched',
                'choices': [
                    'First (1) Choice', 'Second (2)  Choice', 'Third (3) Choice', 
                    'Fourth (4) Choice', 'Fifth (5) Choice'
                ],
                'teammates': [
                    'Team Member #1 UW NetID', 'Team Member #2 UW NetID', 'Team Member #3 UW NetID'
                ]
            },
            'team_size': {'min': 4, 'max': 6},
            'weights': [0, 5, 15, 30, 50],
            'unlisted_penalty': 200,
            'teammate_penalty': 50
        }

    students = []
    projects_set = set()
    col = config['column_map']
    
    for row in rows:
        choices = [row.get(c) for c in col['choices'] if row.get(c)]
        teammates = [row.get(t) for t in col['teammates'] if row.get(t)]
        pitched = row.get(col['pitched'])
        
        student_data = {
            'name': row.get(col['name']),
            'netid': row.get(col['netid']),
            'choices': choices,
            'teammates': teammates,
            'is_pitcher': pitched and choices and (pitched == choices[0])
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

    # Constraints: Team Capacity
    min_size = config['team_size']['min']
    max_size = config['team_size']['max']
    for p in range(num_projects):
        solver.Add(sum(x[s][p] for s in range(num_students)) <= max_size * y[p])
        solver.Add(sum(x[s][p] for s in range(num_students)) >= min_size * y[p])

    # Constraints: Pitcher Requirement
    for s_idx, student in enumerate(students):
        if student['is_pitcher']:
            p_name = student['choices'][0]
            if p_name in project_to_idx:
                p_idx = project_to_idx[p_name]
                solver.Add(x[s_idx][p_idx] == y[p_idx])

    # Objective
    weights = config['weights']
    unlisted_penalty = config['unlisted_penalty']
    teammate_penalty_weight = config['teammate_penalty']
    obj_terms = []

    for s_idx, student in enumerate(students):
        for p_idx, p_name in enumerate(projects):
            if p_name in student['choices']:
                choice_rank = student['choices'].index(p_name)
                # Handle cases where preferences might be fewer than weights defined
                weight = weights[choice_rank] if choice_rank < len(weights) else unlisted_penalty
                obj_terms.append(x[s_idx][p_idx] * weight)
            else:
                obj_terms.append(x[s_idx][p_idx] * unlisted_penalty)

    # Teammate Penalty: z_{s,t,p} >= x[s][p] - x[t][p] and z_{s,t,p} >= x[t][p] - x[s][p]
    netid_to_idx = {s['netid']: i for i, s in enumerate(students) if s['netid']}
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

    status = solver.Solve()

    if status == pywraplp.Solver.OPTIMAL or status == pywraplp.Solver.FEASIBLE:
        results = {}
        for p_idx, p_name in enumerate(projects):
            if y[p_idx].solution_value() > 0.5:
                results[p_name] = [students[s_idx]['name'] for s_idx in range(num_students) if x[s_idx][p_idx].solution_value() > 0.5]
        return results
    else:
        return None
