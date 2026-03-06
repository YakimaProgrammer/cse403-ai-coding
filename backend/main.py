import io
import csv
from fastapi import FastAPI, UploadFile, File, HTTPException
from backend.assignment_solver import solve_assignments_from_list

app = FastAPI()

REQUIRED_COLUMNS = [
    "Name", "NetID", "Project Pitched", "First (1) Choice", 
    "Second (2)  Choice", "Third (3) Choice", "Fourth (4) Choice", 
    "Fifth (5) Choice", "Team Member #1 UW NetID", 
    "Team Member #2 UW NetID", "Team Member #3 UW NetID"
]

@app.post("/solve")
async def solve_teams(file: UploadFile = File(...)):
    if not file.filename.endswith('.csv'):
        raise HTTPException(status_code=400, detail="File must be a CSV.")

    content = await file.read()
    try:
        decoded = content.decode('utf-8')
        reader = csv.DictReader(io.StringIO(decoded))
        
        # Validate headers (Simple check for NetID and Name)
        if 'NetID' not in reader.fieldnames or 'Name' not in reader.fieldnames:
            raise HTTPException(status_code=400, detail="CSV must at least contain 'NetID' and 'Name' columns.")
        
        rows = list(reader)
        if not rows:
            raise HTTPException(status_code=400, detail="CSV is empty.")

        result = solve_assignments_from_list(rows)
        
        if result is None:
            raise HTTPException(status_code=422, detail="No feasible solution found with given constraints.")
        
        return result

    except Exception as e:
        if isinstance(e, HTTPException):
            raise e
        raise HTTPException(status_code=500, detail=f"Internal error processing file: {str(e)}")
