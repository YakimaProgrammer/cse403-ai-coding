# CSE403 Team Assignment Solver

An automated tool to assign students to project teams based on their ranked preferences and teammate requests using linear programming (OR-Tools).

## Live Demo
You can try the application here: [https://magnusfulton.com/cse403/project-assignment/](https://magnusfulton.com/cse403/project-assignment/)

Reference data for testing is available at: [GenAI-InputFile - ProjectPreferences.csv](https://magnusfulton.com/cse403/project-assignment/GenAI-InputFile%20-%20ProjectPreferences.csv)

## Installation & Build

### Using Docker (Recommended)
The project is containerized to handle both the React frontend build and the Python FastAPI backend.

1. **Build the image**:
   ```bash
   docker build -t team-solver --build-arg API_BASE_PATH=/cse403/project-assignment/ .
   ```
   *Note: Set `API_BASE_PATH` to the subpath where you intend to serve the app, or leave it empty for root.*

2. **Run the container**:
   ```bash
   docker run -p 8000:8000 team-solver
   ```

### Local Development

#### Backend
1. Navigate to the `backend` directory.
2. Install dependencies:
   ```bash
   pip install -r requirements.txt
   ```
3. Run the server:
   ```bash
   uvicorn main:app --reload
   ```

#### Frontend
1. Navigate to the `frontend` directory.
2. Install dependencies:
   ```bash
   npm install
   ```
3. Start the development server:
   ```bash
   npm run dev
   ```

## Usage
1. Prepare a CSV file containing student preferences (following the format in the reference data).
2. Upload the CSV via the web interface.
3. (Optional) Adjust weights for choice ranks, team size constraints, and penalties in the "More Options" menu.
4. Click "Solve Team Assignments" to generate the optimal groupings.
