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

    # Constraints: Team Capacity
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

    status = solver.Solve()

    if status == pywraplp.Solver.OPTIMAL or status == pywraplp.Solver.FEASIBLE:
        results = {}
        for p_idx, p_name in enumerate(projects):
            if y[p_idx].solution_value() > 0.5:
                results[p_name] = [students[s_idx]['name'] for s_idx in range(num_students) if x[s_idx][p_idx].solution_value() > 0.5]
        return results
    else:
        return None
