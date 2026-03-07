import io
import csv
import json
import os
from fastapi import FastAPI, UploadFile, File, Form, HTTPException, APIRouter
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles
from assignment_solver import solve_assignments_from_list

app = FastAPI(title="CSE403 Team Assignment Solver")
router = APIRouter()

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

REQUIRED_COLUMNS = [
    "Name", "NetID", "Project Pitched", "First (1) Choice",
    "Second (2)  Choice", "Third (3) Choice", "Fourth (4) Choice",
    "Fifth (5) Choice", "Team Member #1 UW NetID",
    "Team Member #2 UW NetID", "Team Member #3 UW NetID"
]


@router.get("/health")
async def health():
    return {"status": "ok"}


@router.post("/solve")
async def solve_teams(file: UploadFile = File(...), options: str = Form("{}")):
    if not file.filename.endswith('.csv'):
        raise HTTPException(status_code=400, detail="File must be a CSV.")

    content = await file.read()
    try:
        decoded = content.decode('utf-8')
        reader = csv.DictReader(io.StringIO(decoded))

        # Validate headers
        if not all(col in reader.fieldnames for col in REQUIRED_COLUMNS):
            raise HTTPException(status_code=400, detail="CSV is missing required columns.")

        rows = list(reader)
        if not rows:
            raise HTTPException(status_code=400, detail="CSV is empty.")

        try:
            parsed_options = json.loads(options)
        except:
            parsed_options = {}

        result = solve_assignments_from_list(rows, parsed_options)

        if result is None:
            raise HTTPException(status_code=422, detail="No feasible solution found with given constraints.")

        return result

    except Exception as e:
        if isinstance(e, HTTPException):
            raise e
        raise HTTPException(status_code=500, detail=f"Internal error processing file: {str(e)}")

# Determine base path from environment variable
BASE_PATH = os.getenv("API_BASE_PATH", "")

# Serve static files if they exist (built by Docker)
if os.path.exists("./static_build"):
    app.mount(BASE_PATH + "/" if BASE_PATH else "/", StaticFiles(directory="./static_build", html=True), name="static")

# Include the router with the prefix
app.include_router(router, prefix=BASE_PATH)
