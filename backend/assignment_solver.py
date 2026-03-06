import csv
from ortools.linear_solver import pywraplp

def solve_assignments(csv_file_path):
    with open(csv_file_path, mode='r', encoding='utf-8') as f:
        reader = csv.DictReader(f)
        return solve_assignments_from_list(list(reader))

def solve_assignments_from_list(rows):
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
        
        # Preference for size 6: small negative weight (reward) for size 6
        is_size_6 = solver.BoolVar(f'is_size_6_{p}')
        solver.Add(team_size >= 6 * is_size_6)
        obj_terms.append(is_size_6 * -1)

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

    # Teammate Requirement: If a student lists teammates, they must be with at least one.
    netid_to_idx = {s['netid']: i for i, s in enumerate(students)}
    for s_idx, student in enumerate(students):
        valid_teammate_indices = [netid_to_idx[t] for t in student['teammates'] if t in netid_to_idx and netid_to_idx[t] != s_idx]
        if valid_teammate_indices:
            for p_idx in range(num_projects):
                # x[s][p] <= sum(x[t][p] for t in teammates)
                # If student s is on project p, at least one teammate t must also be on project p.
                solver.Add(x[s_idx][p_idx] <= sum(x[t_idx][p_idx] for t_idx in valid_teammate_indices))

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
