# Questions

## 1. What are your names and UW NetIds?
Magnus Fulton (mjfulton)  
Ezra Amerga (ezra27)  

## 2. Which GenAI assistant(s) did you use, and how did you use it to complete your task?
Magnus used `gemini-3-flash-preview` via the [`aider`](https://aider.chat/) command line tool. I used in a couple of ways.
First, I would prompt the model with high level thoughts and tool specifications:

> Let's solve a fun problem with Python's ortools!  
>   
> Consider the following subset of some data in "backend/GenAI-InputFile\ -\ ProjectPreferences.csv"  
>   
> We want to assign students to "teams".  
> Project teams are 5-6 students, with a preferred size of 6. If necessary, teams can be as small as 4 people, but no smaller.  
> The app should assign students first to a project on their top 5 list (in that order!), but may assign students to any team if needed.  
> Not every project will be chosen.  
> Students who pitched a project must be assigned to that project if it is selected and if it's their #1 choice.  
> Students are assigned tp [rpjects with at least one of their preferred teammates.  
>   
> This is a linear programming problem at its core. I think we can solve it with an integer program. Let's start designing it in Python.  
>   
> Here's some reference data to help, as well.  
>   
>   
> Name,NetID,Project Pitched,First (1) Choice,Second (2)  Choice,Third (3) Choice,Fourth (4) Choice,Fifth (5) Choice,Team Member #1 UW NetID,Team Member #2 UW NetID,Team Member #3 UW NetID  
> Student 1,student1,Project 1,Project 1,Project 2,Project 3,Project 4,Project 5,student2,,  
> Student 2,student2,Project 1,Project 1,Project 6,Project 7,Project 3,Project 8,student1,,  
> Student 3,student3,Project 10,Project 1,Project 9,Project 10,Project 8,Project 11,student3,,  
> Student 4,student4,Project 14,Project 14,Project 13,Project 2,Project 4,Project 12,student4,,  
> Student 5,student5,Project 14,Project 14,Project 13,Project 2,Project 4,Project 12,student4,,  
> Student 6,student6,Project 24,Project 24,Project 23,Project 10,Project 20,Project 11,student5,,  
> Student 7,student7,,Project 23,Project 28,Project 4,Project 18,Project 14,student7,,  
> Student 8,student8,Project 23,Project 23,Project 24,Project 12,Project 10,Project 13,student9,student8,  
> Student 9,student9,Project 23,Project 23,Project 24,Project 12,Project 10,Project 13,student9,student8,  
> Student 10,student10,Project 20,Project 23,Project 20,Project 19,Project 13,Project 28,aidanyu,,  
> Student 11,student11,Project 19,Project 19,Project 6,Project 9,Project 8,Project 13,student11,student12,student13  
> Student 12,student12,Project 19,Project 19,Project 6,Project 9,Project 8,Project 13,student13,student11,student12  
> Student 13,student13,Project 19,Project 19,Project 6,Project 9,Project 8,Project 13,student11,student13,student12  

Here is the model's response:

> To implement this student-to-project assignment using OR-Tools, we will model this as a Mixed-Integer Programming (MIP) problem.
> 
> ### Architecture Plan
> 1.  **Variables**: Create a boolean matrix $x[s, p]$ where $s$ is a student and $p$ is a project ($1$ if student is assigned to project, $0$ otherwise). Create a boolean variable $y[p]$ to indicate if a project is "active".
> 2.  **Constraints**:
>     *   **Single Assignment**: Each student must be assigned to exactly one project.
>     *   **Team Capacity**: If $y[p] = 1$, the sum of $x[s, p]$ must be between 4 and 6. If $y[p] = 0$, the sum must be 0.
>     *   **Pitcher Rule**: If student $s$ pitched project $p$ and listed it as choice #1, then $x[s, p] = y[p]$. (If the project is active, they must be on it).
>     *   **Preferred Teammate**: For each student $s$ and their preferred teammate $t$, if $s$ is assigned to project $p$, then $t$ should ideally be on project $p$.
> 3.  **Objective**: Minimize "cost" where costs are assigned to preferences (e.g., Choice 1 = 0 cost, Choice 2 = 10, Choice 3 = 20, ..., Unlisted = 100) and penalties are added for breaking teammate pairings.
> 
> ### Instructions for Editor Engineer
> 
> Create a new file `backend/assignment_solver.py` and implement the following logic:
> 
> 1.  **Data Ingestion**:
>     *   Parse the CSV into a list of students and a unique list of projects.
>     *   Map project names to indices.
>     *   Store student preferences (indices 1-5) and preferred teammate NetIDs.
> 
> 2.  **Solver Setup**:
>     *   Use `pywraplp.Solver.CreateSolver('SCIP')`.
> 
> 3.  **Variable Definitions**:
>     *   `x[s][p]`: `solver.BoolVar(f'x_{s}_{p}')`
>     *   `y[p]`: `solver.BoolVar(f'y_{p}')`
> 
> 4.  **Constraint Implementation**:
>     *   **Assignment**: `solver.Add(sum(x[s][p] for p in projects) == 1)` for every student `s`.
>     *   **Conditional Capacity**: For every project `p`:
>         *   `solver.Add(sum(x[s][p] for s in students) <= 6 * y[p])`
>         *   `solver.Add(sum(x[s][p] for s in students) >= 4 * y[p])`
>     *   **Pitcher Requirement**: For each student $s$ who pitched project $p$ as their #1 choice:
>         *   `solver.Add(x[s][p] == y[p])`
> 
> 5.  **Objective Function**:
>     *   Assign weights: Choice 1: 0, Choice 2: 5, Choice 3: 15, Choice 4: 30, Choice 5: 50, Not on list: 200.
>     *   Add a **Teammate Penalty**: For every student $s$ and their requested teammate $t$, add a penalty to the objective if $x[s][p] \neq x[t][p]$. Since this is non-linear, represent it by adding a variable $z_{s,t}$ and constraints: `z_{s,t} >= x[s][p] - x[t][p]` and `z_{s,t} >= x[t][p] - x[s][p]`. Alternatively, simplify by rewarding pairs on the same project.
>     *   Set `solver.Minimize(total_cost)`.
> 
> 6.  **Execution**:
>     *   Call `solver.Solve()`.
>     *   Output a mapping of Project Names to lists of Student Names/NetIDs.

Here, I didn't bother going any more in-depth into the design or architecture because ultimately we were writing a single function exposed to the internet via FastAPI. You can see that I started smaller and told the model to just focus on the linear programming side of the problem.  
I also used `aider`/`Gemini` to implement new features as well:
> Could we even make the team sizes be a tunable parameter? For example - allowing groups of up to N and mins of K?  

> ```javascript
> frontend/src/App.jsx
> <<<<<<< SEARCH
>   const [options, setOptions] = useState({
>     w1: 0, w2: 5, w3: 15, w4: 30, w5: 50,
>     unlisted: 200, teammate: 50,
>     size6: -1, size5: 25, size4: 50
>   })
> =======
>   const [options, setOptions] = useState({
>     w1: 0, w2: 5, w3: 15, w4: 30, w5: 50,
>     unlisted: 200, teammate: 50,
>     size6: -1, size5: 25, size4: 50,
>     minSize: 4, maxSize: 6
>   })
> >>>>>>> REPLACE
> ```
> 
> ```javascript
> frontend/src/App.jsx
> <<<<<<< SEARCH
>             <div className="option-group">
>               <h4>Team Size Penaltes</h4>
>               <label>Size 6: <input type="number" name="size6" value={options.size6} onChange={handleOptionChange} /></label>
> =======
>             <div className="option-group">
>               <h4>Team Size Constraints</h4>
>               <label>Min Size: <input type="number" name="minSize" value={options.minSize} onChange={handleOptionChange} /></label>
>               <label>Max Size: <input type="number" name="maxSize" value={options.maxSize} onChange={handleOptionChange} /></label>
>               <hr />
>               <h4>Team Size Penalties</h4>
>               <label>Size 6: <input type="number" name="size6" value={options.size6} onChange={handleOptionChange} /></label>
> >>>>>>> REPLACE
> ```
> 
> ```python
> backend/assignment_solver.py
> <<<<<<< SEARCH
>     # Constraints: Team Capacity
>     for p in range(num_projects):
>         team_size = sum(x[s][p] for s in range(num_students))
>         solver.Add(team_size <= 6 * y[p])
>         solver.Add(team_size >= 4 * y[p])
> =======
>     # Constraints: Team Capacity
>     min_team_size = options.get('minSize', 4)
>     max_team_size = options.get('maxSize', 6)
> 
>     for p in range(num_projects):
>         team_size = sum(x[s][p] for s in range(num_students))
>         solver.Add(team_size <= max_team_size * y[p])
>         solver.Add(team_size >= min_team_size * y[p])
> >>>>>>> REPLACE
> ```
> 
> ```python
> backend/assignment_solver.py
> <<<<<<< SEARCH
>         # Link size variables to actual team_size
>         solver.Add(team_size >= 6 * is_size_6 + 5 * is_size_5 + 4 * is_size_4)
>         solver.Add(team_size <= 6 * is_size_6 + 5 * is_size_5 + 4 * is_size_4 + 6 * (1 - y[p]))
> =======
>         # Link size variables to actual team_size
>         solver.Add(team_size >= 6 * is_size_6 + 5 * is_size_5 + 4 * is_size_4)
>         solver.Add(team_size <= 6 * is_size_6 + 5 * is_size_5 + 4 * is_size_4 + max_team_size * (1 - y[p]))
> >>>>>>> REPLACE
> ```

I also had some back and forth with the model while verifying code output and doing bug fixes:
> Close! We should be able to assign penalties for all group sizes, no matter what sizes we allow.

> I will refactor the team size penalty logic in both the frontend and backend to support arbitrary team sizes. Instead of hardcoded size variables, the backend will now iterate from `minSize` to `maxSize` and apply penalties from a provided mapping, and the frontend will allow users to define these penalties dynamically.
> 
> ```javascript
> frontend/src/App.jsx
> <<<<<<< SEARCH
>     w1: 0, w2: 5, w3: 15, w4: 30, w5: 50,
>     unlisted: 200, teammate: 50,
>     size6: -1, size5: 25, size4: 50,
>     minSize: 4, maxSize: 6
>   })
> =======
>     w1: 0, w2: 5, w3: 15, w4: 30, w5: 50,
>     unlisted: 200, teammate: 50,
>     sizePenalties: { 4: 50, 5: 25, 6: -1 },
>     minSize: 4, maxSize: 6
>   })
> >>>>>>> REPLACE
> ```
> 
> ```javascript
> frontend/src/App.jsx
> <<<<<<< SEARCH
>   const handleOptionChange = (e) => {
>     setOptions({ ...options, [e.target.name]: parseInt(e.target.value) || 0 })
>   }
> =======
>   const handleOptionChange = (e) => {
>     const { name, value } = e.target
>     if (name.startsWith('sizeP-')) {
>       const size = name.split('-')[1]
>       setOptions({
>         ...options,
>         sizePenalties: { ...options.sizePenalties, [size]: parseInt(value) || 0 }
>       })
>     } else {
>       setOptions({ ...options, [name]: parseInt(value) || 0 })
>     }
>   }
> >>>>>>> REPLACE
> ```
> 
> ```javascript
> frontend/src/App.jsx
> <<<<<<< SEARCH
>               <label>Max Size: <input type="number" name="maxSize" value={options.maxSize} onChange={handleOptionChange} /></label>
>               <hr />
>               <h4>Team Size Penalties</h4>
>               <label>Size 6: <input type="number" name="size6" value={options.size6} onChange={handleOptionChange} /></label>
>               <label>Size 5: <input type="number" name="size5" value={options.size5} onChange={handleOptionChange} /></label>
>               <label>Size 4: <input type="number" name="size4" value={options.size4} onChange={handleOptionChange} /></label>
>               <hr />
> =======
>               <label>Max Size: <input type="number" name="maxSize" value={options.maxSize} onChange={handleOptionChange} /></label>
>               <hr />
>               <h4>Team Size Penalties</h4>
>               {Array.from({ length: options.maxSize - options.minSize + 1 }, (_, i) => options.minSize + i).map(size => (
>                 <label key={size}>
>                   Size {size}: 
>                   <input 
>                     type="number" 
>                     name={`sizeP-${size}`} 
>                     value={options.sizePenalties[size] ?? 0} 
>                     onChange={handleOptionChange} 
>                   />
>                 </label>
>               ))}
>               <hr />
> >>>>>>> REPLACE
> ```
> 
> ```python
> backend/assignment_solver.py
> <<<<<<< SEARCH
>         # Team Size Penalties/Rewards
>         is_size_6 = solver.BoolVar(f'is_size_6_{p}')
>         is_size_5 = solver.BoolVar(f'is_size_5_{p}')
>         is_size_4 = solver.BoolVar(f'is_size_4_{p}')
> 
>         # Ensure exactly one size variable is true if the project is active
>         solver.Add(is_size_6 + is_size_5 + is_size_4 == y[p])
>         
>         # Link size variables to actual team_size
>         solver.Add(team_size >= 6 * is_size_6 + 5 * is_size_5 + 4 * is_size_4)
>         solver.Add(team_size <= 6 * is_size_6 + 5 * is_size_5 + 4 * is_size_4 + max_team_size * (1 - y[p]))
> 
>         # Penalties: Customizable via options
>         obj_terms.append(is_size_6 * options.get('size6', -1))
>         obj_terms.append(is_size_5 * options.get('size5', 25))
>         obj_terms.append(is_size_4 * options.get('size4', 50))
> =======
>         # Team Size Penalties/Rewards for all valid sizes
>         size_vars = []
>         size_penalties = options.get('sizePenalties', {})
>         
>         for size in range(min_team_size, max_team_size + 1):
>             is_size_k = solver.BoolVar(f'is_size_{size}_{p}')
>             size_vars.append((size, is_size_k))
>             
>             penalty = size_penalties.get(str(size), 0)
>             obj_terms.append(is_size_k * penalty)
> 
>         # Ensure exactly one size variable is true if the project is active
>         solver.Add(sum(v for _, v in size_vars) == y[p])
>         
>         # Link size variables to actual team_size
>         actual_size_expr = sum(size * v for size, v in size_vars)
>         solver.Add(team_size >= actual_size_expr)
>         solver.Add(team_size <= actual_size_expr + max_team_size * (1 - y[p]))
> >>>>>>> REPLACE
> ```

## 3. Describe how well GenAI performed on your task. Consider how many prompts it took to get a working solution, how many suggestions were useful, and how many required significant human interventions. Did you have to write any code yourself? If so, how much?

I sent approximately 70 prompts to aider. While I wrote a handful of changes myself, I also tried to lean really hard into using the tool, especially since I don't typically write with AI code generation tools. I found that most suggestions by the LLM were generally pretty helpful, but sometimes it got lost in the weeds and would forget what we were working on. I also found sometimes the model would also get stuck on problems as well, which definately kept the onus on me as a developer to think more widely about the problem and explore other approaches. There were also quite a few changes proposed by `aider` that I rejected as well. I'm not sure the exact count because the built-in chat tool's command - `/undo` - drops the commit from the history, rather than `git revert`'ing it.

## 4. Did you provide the AI with the full requirements at once or build it piece-by-piece? What worked better? At any point, did the AI provide a solution that looked correct but was logically flawed? If so, how did you catch it?

Magnus: I built the app piece by piece, largely off of a design in my head and a quick sketch I made on some paper. I noticed the model would give incomplete results and get lost if I tried to have it do too much at once. Interestingly enough, at least the initial iterations of the [`ortools`](https://pypi.org/project/ortools/) were largely correct, which was surprising, but also expected as it was mostly procedural translating the specification's requirements into mathematical notation.

## 5. Describe specific moments where GenAI sped up your development process and moments where it slowed you down. What patterns did you notice about when GenAI was most and least helpful?

Magnus: `aider` was a significant typing accelerator for me since I already knew largely how to use `ortools`, write API code in Python, and write React code. Since these elements didn't require much planning, I was able to spend a lot more time rapidly iterating on the design and tweaking things. Normally, this would've happened on paper or a whiteboard for me. It also let me explore some ideas as well. If you take a look at [`63e5b48`](https://github.com/YakimaProgrammer/cse403-ai-coding/tree/63e5b48be1e9473830314cea1ceb464c741e7aea) (and earlier!) you can see where I used AI to rapidly prototype a client-side only solution based on Rust and WebAssembly so we wouldn't even need a backend server. That said, since I could rapidly iterate so much, it encouraged me to fixate on a problem (even more than I'm already inclined to do) longer than I otherwise would've, which slowed me down.

## 6. Based on this short experience, how might widespread GenAI adoption change software engineering, and what new skills will engineers need?

Magnus: I think that GenAI is definitely reinforcing the idea that being a software dev is not about how many languages you know or libraries you've memorized - it's about problem-solving and critical thinking. This was consistently where I'd have to step in and guide the model whenever it lost the plot. That said, raw code writing is often a very newgrad role to have as engineers move into more serious roles in the tech industry, which can make it hard for newgrads to get a foothold in the industry.
